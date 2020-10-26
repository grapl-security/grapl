import base64
import hashlib
import inspect
import json
import logging
import os
import sys
import traceback


from collections import defaultdict
from datetime import datetime
from multiprocessing import Process, Pipe
from multiprocessing.connection import Connection
from multiprocessing.pool import ThreadPool
from pathlib import Path
from typing import Any, Optional, Tuple, List, Dict, Iterator, Union

import boto3  # type: ignore
import redis
from grapl_analyzerlib.analyzer import Analyzer
from grapl_analyzerlib.execution import ExecutionHit, ExecutionComplete, ExecutionFailed
from grapl_analyzerlib.nodes.base import BaseView
from grapl_analyzerlib.queryable import Queryable
from grapl_analyzerlib.subgraph_view import SubgraphView
from grapl_analyzerlib.plugin_retriever import load_plugins
from pydgraph import DgraphClientStub, DgraphClient  # type: ignore

MODEL_PLUGINS_DIR = os.environ.get("MODEL_PLUGINS_DIR", "/tmp")
sys.path.insert(0, MODEL_PLUGINS_DIR)

IS_LOCAL = bool(os.environ.get("IS_LOCAL", False))
IS_RETRY = os.environ["IS_RETRY"]

GRAPL_LOG_LEVEL = os.getenv("GRAPL_LOG_LEVEL")
LEVEL = "ERROR" if GRAPL_LOG_LEVEL is None else GRAPL_LOG_LEVEL
LOGGER = logging.getLogger(__name__)
LOGGER.setLevel(LEVEL)
LOGGER.addHandler(logging.StreamHandler(stream=sys.stdout))

try:
    directory = Path(MODEL_PLUGINS_DIR + "/model_plugins/")
    directory.mkdir(parents=True, exist_ok=True)
except Exception as e:
    LOGGER.error("Failed to create directory", e)


class NopCache(object):
    def set(self, key, value):
        pass

    def get(self, key):
        return False


EitherCache = Union[NopCache, redis.Redis]

if IS_LOCAL:
    message_cache: EitherCache = NopCache()
    hit_cache: EitherCache = NopCache()
else:
    MESSAGECACHE_ADDR = os.environ["MESSAGECACHE_ADDR"]
    MESSAGECACHE_PORT = int(os.environ["MESSAGECACHE_PORT"])

    HITCACHE_ADDR = os.environ["HITCACHE_ADDR"]
    HITCACHE_PORT = os.environ["HITCACHE_PORT"]

    message_cache = redis.Redis(host=MESSAGECACHE_ADDR, port=MESSAGECACHE_PORT, db=0)
    hit_cache = redis.Redis(host=HITCACHE_ADDR, port=int(HITCACHE_PORT), db=0)


def parse_s3_event(s3, event) -> str:
    bucket = event["s3"]["bucket"]["name"]
    key = event["s3"]["object"]["key"]
    return download_s3_file(s3, bucket, key)


def download_s3_file(s3, bucket: str, key: str) -> str:
    obj = s3.Object(bucket, key)
    return obj.get()["Body"].read()


def is_analyzer(analyzer_name, analyzer_cls):
    if analyzer_name == "Analyzer":  # This is the base class
        return False
    return (
        hasattr(analyzer_cls, "get_queries")
        and hasattr(analyzer_cls, "build")
        and hasattr(analyzer_cls, "on_response")
    )


def get_analyzer_objects(dgraph_client: DgraphClient) -> Dict[str, Analyzer]:
    clsmembers = inspect.getmembers(sys.modules[__name__], inspect.isclass)
    return {
        an[0]: an[1].build(dgraph_client)
        for an in clsmembers
        if is_analyzer(an[0], an[1])
    }


def check_caches(
    file_hash: str, msg_id: str, node_key: str, analyzer_name: str
) -> bool:
    if check_msg_cache(file_hash, node_key, msg_id):
        LOGGER.debug("cache hit - already processed")
        return True

    if check_hit_cache(analyzer_name, node_key):
        LOGGER.debug("cache hit - already matched")
        return True

    return False


def exec_analyzers(
    dg_client,
    file: str,
    msg_id: str,
    nodes: List[BaseView],
    analyzers: Dict[str, Analyzer],
    sender: Any,
):
    if not analyzers:
        LOGGER.warning("Received empty dict of analyzers")
        return

    if not nodes:
        LOGGER.warning("Received empty array of nodes")

    for node in nodes:
        querymap: Dict[str, List[Queryable]] = defaultdict(list)

        for an_name, analyzer in analyzers.items():
            if check_caches(file, msg_id, node.node_key, an_name):
                continue

            queries = analyzer.get_queries()
            if isinstance(queries, list) or isinstance(queries, tuple):
                querymap[an_name].extend(queries)
            else:
                querymap[an_name].append(queries)

        for an_name, queries in querymap.items():
            analyzer = analyzers[an_name]

            for _, query in enumerate(queries):
                response = query.query_first(dg_client, contains_node_key=node.node_key)
                if response:
                    LOGGER.debug(f"Found a hit for analyzer {an_name}, executing on_response()")
                    analyzer.on_response(response, sender)


def chunker(seq, size):
    return [seq[pos : pos + size] for pos in range(0, len(seq), size)]


def mg_alphas() -> Iterator[Tuple[str, int]]:
    mg_alphas = os.environ["MG_ALPHAS"].split(",")
    for mg_alpha in mg_alphas:
        host, port = mg_alpha.split(":")
        yield host, int(port)


def execute_file(name: str, file: str, graph: SubgraphView, sender, msg_id):
    try:
        pool = ThreadPool(processes=4)

        exec(file, globals())
        client_stubs = (
            DgraphClientStub(f"{host}:{port}") for host, port in mg_alphas()
        )
        client = DgraphClient(*client_stubs)

        analyzers = get_analyzer_objects(client)
        if not analyzers:
            LOGGER.warning(f"Got no analyzers for file: {name}")

        LOGGER.info(f"Executing analyzers: {[an for an in analyzers.keys()]}")

        chunk_size = 100

        if IS_RETRY == "True":
            chunk_size = 10

        for nodes in chunker([n for n in graph.node_iter()], chunk_size):
            LOGGER.info(f"Querying {len(nodes)} nodes")

            def exec_analyzer(nodes, sender):
                try:
                    exec_analyzers(client, file, msg_id, nodes, analyzers, sender)

                    return nodes
                except Exception as e:
                    LOGGER.error(traceback.format_exc())
                    LOGGER.error(f"Execution of {name} failed with {e} {e.args}")
                    sender.send(ExecutionFailed())
                    raise

            exec_analyzer(nodes, sender)
            pool.apply_async(exec_analyzer, args=(nodes, sender))

        pool.close()

        pool.join()

        sender.send(ExecutionComplete())

    except Exception as e:
        LOGGER.error(traceback.format_exc())
        LOGGER.error(f"Execution of {name} failed with {e} {e.args}")
        sender.send(ExecutionFailed())
        raise


def emit_event(s3, event: ExecutionHit) -> None:
    LOGGER.info(f"emitting event for: {event.analyzer_name, event.nodes}")

    event_s = json.dumps(
        {
            "nodes": json.loads(event.nodes),
            "edges": json.loads(event.edges),
            "analyzer_name": event.analyzer_name,
            "risk_score": event.risk_score,
            "lenses": event.lenses,
            "risky_node_keys": event.risky_node_keys,
        }
    )
    event_hash = hashlib.sha256(event_s.encode())
    key = base64.urlsafe_b64encode(event_hash.digest()).decode("utf-8")

    obj = s3.Object(
        f"{os.environ['BUCKET_PREFIX']}-analyzer-matched-subgraphs-bucket", key
    )
    obj.put(Body=event_s)

    if IS_LOCAL:
        sqs = boto3.client(
            "sqs",
            region_name="us-east-1",
            endpoint_url="http://sqs.us-east-1.amazonaws.com:9324",
            aws_access_key_id="dummy_cred_aws_access_key_id",
            aws_secret_access_key="dummy_cred_aws_secret_access_key",
        )
        send_s3_event(
            sqs,
            "http://sqs.us-east-1.amazonaws.com:9324/queue/grapl-engagement-creator-queue",
            "local-grapl-analyzer-matched-subgraphs-bucket",
            key,
        )


def check_msg_cache(file: str, node_key: str, msg_id: str) -> bool:
    to_hash = str(file) + str(node_key) + str(msg_id)
    event_hash = hashlib.sha256(to_hash.encode()).hexdigest()
    return bool(message_cache.get(event_hash))


def update_msg_cache(file: str, node_key: str, msg_id: str) -> None:
    to_hash = str(file) + str(node_key) + str(msg_id)
    event_hash = hashlib.sha256(to_hash.encode()).hexdigest()
    message_cache.set(event_hash, "1")


def check_hit_cache(file: str, node_key: str) -> bool:
    to_hash = str(file) + str(node_key)
    event_hash = hashlib.sha256(to_hash.encode()).hexdigest()
    return bool(hit_cache.get(event_hash))


def update_hit_cache(file: str, node_key: str) -> None:
    to_hash = str(file) + str(node_key)
    event_hash = hashlib.sha256(to_hash.encode()).hexdigest()
    hit_cache.set(event_hash, "1")


def lambda_handler_fn(events: Any, context: Any) -> None:
    # Parse sns message
    LOGGER.debug(f"handling events: {events} context: {context}")

    client_stubs = (DgraphClientStub(f"{host}:{port}") for host, port in mg_alphas())
    client = DgraphClient(*client_stubs)

    s3 = get_s3_client()

    load_plugins(os.environ["BUCKET_PREFIX"], s3, os.path.abspath(MODEL_PLUGINS_DIR))

    for event in events["Records"]:
        if not IS_LOCAL:
            event = json.loads(event["body"])["Records"][0]
        data = parse_s3_event(s3, event)

        message = json.loads(data)

        LOGGER.info(f'Executing Analyzer: {message["key"]}')
        analyzer = download_s3_file(
            s3, f"{os.environ['BUCKET_PREFIX']}-analyzers-bucket", message["key"]
        )
        analyzer_name = message["key"].split("/")[-2]

        subgraph = SubgraphView.from_proto(client, bytes(message["subgraph"]))

        # TODO: Validate signature of S3 file
        LOGGER.info(f"event {event}")
        rx, tx = Pipe(duplex=False)  # type: Tuple[Connection, Connection]
        p = Process(
            target=execute_file, args=(analyzer_name, analyzer, subgraph, tx, "")
        )

        p.start()
        t = 0

        while True:
            p_res = rx.poll(timeout=5)
            if not p_res:
                t += 1
                LOGGER.info(
                    f"Polled {analyzer_name} for {t * 5} seconds without result"
                )
                continue
            result: Optional[Any] = rx.recv()

            if isinstance(result, ExecutionComplete):
                LOGGER.info("execution complete")
                break

            # emit any hits to an S3 bucket
            if isinstance(result, ExecutionHit):
                LOGGER.info(
                    f"emitting event for {analyzer_name} {result.analyzer_name} {result.root_node_key}"
                )
                emit_event(s3, result)
                update_msg_cache(analyzer, result.root_node_key, message["key"])
                update_hit_cache(analyzer_name, result.root_node_key)

            assert not isinstance(
                result, ExecutionFailed
            ), f"Analyzer {analyzer_name} failed."

        p.join()


### LOCAL HANDLER


def into_sqs_message(bucket: str, key: str) -> str:
    return json.dumps(
        {
            "Records": [
                {
                    "eventTime": datetime.utcnow().isoformat(),
                    "principalId": {
                        "principalId": None,
                    },
                    "requestParameters": {
                        "sourceIpAddress": None,
                    },
                    "responseElements": {},
                    "s3": {
                        "schemaVersion": None,
                        "configurationId": None,
                        "bucket": {
                            "name": bucket,
                            "ownerIdentity": {
                                "principalId": None,
                            },
                        },
                        "object": {
                            "key": key,
                            "size": 0,
                            "urlDecodedKey": None,
                            "versionId": None,
                            "eTag": None,
                            "sequencer": None,
                        },
                    },
                }
            ]
        }
    )


def send_s3_event(
    sqs_client: Any,
    queue_url: str,
    output_bucket: str,
    output_path: str,
):
    sqs_client.send_message(
        QueueUrl=queue_url,
        MessageBody=into_sqs_message(
            bucket=output_bucket,
            key=output_path,
        ),
    )


def get_s3_client():
    if IS_LOCAL:
        return boto3.resource(
            "s3",
            endpoint_url="http://s3:9000",
            aws_access_key_id="minioadmin",
            aws_secret_access_key="minioadmin",
        )

    else:
        return boto3.resource("s3")
