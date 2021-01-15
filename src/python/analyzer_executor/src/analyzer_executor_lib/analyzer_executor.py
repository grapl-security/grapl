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
from multiprocessing import Pipe, Process
from multiprocessing.connection import Connection
from multiprocessing.pool import ThreadPool
from pathlib import Path
from typing import Any, Dict, List, Optional, Union

import boto3  # type: ignore
import redis
from grapl_common.metrics.metric_reporter import MetricReporter, TagPair

from grapl_analyzerlib.analyzer import Analyzer
from grapl_analyzerlib.execution import ExecutionComplete, ExecutionFailed, ExecutionHit
from grapl_analyzerlib.grapl_client import GraphClient
from grapl_analyzerlib.nodes.base import BaseView
from grapl_analyzerlib.plugin_retriever import load_plugins
from grapl_analyzerlib.queryable import Queryable
from grapl_analyzerlib.subgraph_view import SubgraphView

# Set up logger (this is for the whole file, including static methods)
LOGGER = logging.getLogger(__name__)
LOGGER.setLevel(os.getenv("GRAPL_LOG_LEVEL", "ERROR"))
LOGGER.addHandler(logging.StreamHandler(stream=sys.stdout))

# Set up plugins dir for models
MODEL_PLUGINS_DIR = os.getenv("MODEL_PLUGINS_DIR", "/tmp")
sys.path.insert(0, MODEL_PLUGINS_DIR)

# Ensure plugins dir exists
try:
    directory = Path(MODEL_PLUGINS_DIR + "/model_plugins/")
    directory.mkdir(parents=True, exist_ok=True)
except Exception as e:
    LOGGER.error("Failed to create model plugins directory", e)


# TODO:  move generic cache stuff into its own utility file
class NopCache(object):
    def set(self, key, value):
        pass

    def get(self, key):
        return False


EitherCache = Union[NopCache, redis.Redis]


class AnalyzerExecutor:

    # constants
    CHUNK_SIZE_RETRY: int = 10
    CHUNK_SIZE_DEFAULT: int = 100

    # singleton
    _singleton = None

    def __init__(
        self, message_cache, hit_cache, chunk_size, is_local, logger, metric_reporter
    ):
        self.message_cache = message_cache
        self.hit_cache = hit_cache
        self.chunk_size = chunk_size
        self.is_local = is_local
        self.logger = logger
        self.metric_reporter = metric_reporter

    @classmethod
    def singleton(cls):
        if not cls._singleton:
            LOGGER.debug("initializing AnalyzerExecutor singleton")
            is_local = bool(
                os.getenv("IS_LOCAL", False)
            )  # TODO move determination to grapl-common

            # If we're retrying, change the chunk size
            is_retry = os.getenv("IS_RETRY", False)
            if is_retry == "True":
                chunk_size = cls.CHUNK_SIZE_RETRY
            else:
                chunk_size = cls.CHUNK_SIZE_DEFAULT

            # Set up message cache
            messagecache_addr = os.getenv("MESSAGECACHE_ADDR")
            messagecache_port = os.getenv("MESSAGECACHE_PORT")
            if messagecache_port:
                try:
                    messagecache_port = int(messagecache_port)
                except (TypeError, ValueError) as ex:
                    LOGGER.error(
                        f"can't connect to redis, MESSAGECACHE_PORT couldn't cast to int"
                    )
                    raise ex

            if messagecache_addr and messagecache_port:
                LOGGER.debug(
                    f"message cache connecting to redis at {messagecache_addr}:{messagecache_port}"
                )
                message_cache = redis.Redis(
                    host=messagecache_addr, port=messagecache_port, db=0
                )
            else:
                LOGGER.error(
                    f"message cache failed connecting to redis | addr:\t{messagecache_addr} | port:\t{messagecache_port}"
                )
                raise ValueError(
                    f"incomplete redis connection details for message cache"
                )

            # Set up hit cache
            hitcache_addr = os.getenv("HITCACHE_ADDR")
            hitcache_port = os.getenv("HITCACHE_PORT")
            if hitcache_port:
                try:
                    hitcache_port = int(hitcache_port)
                except (TypeError, ValueError) as ex:
                    LOGGER.error(
                        f"can't connect to redis, MESSAGECACHE_PORT couldn't cast to int"
                    )
                    raise ex

            if hitcache_addr and hitcache_port:
                LOGGER.debug(
                    f"hit cache connecting to redis at {hitcache_addr}:{hitcache_port}"
                )
                hit_cache = redis.Redis(
                    host=hitcache_addr, port=int(hitcache_port), db=0
                )
            else:
                LOGGER.error(
                    f"hit cache failed connecting to redis | addr:\t{hitcache_addr} | port:\t{hitcache_port}"
                )
                raise ValueError(f"incomplete redis connection details for hit cache")

            metric_reporter = MetricReporter.create("analyzer-executor")
            # retain singleton
            cls._singleton = cls(
                message_cache, hit_cache, chunk_size, is_local, LOGGER, metric_reporter
            )

        return cls._singleton

    def check_caches(
        self, file_hash: str, msg_id: str, node_key: str, analyzer_name: str
    ) -> bool:
        with self.metric_reporter.histogram_ctx("analyzer-executor.check_caches"):
            if self.check_msg_cache(file_hash, node_key, msg_id):
                self.logger.debug("cache hit - already processed")
                return True

            if self.check_hit_cache(analyzer_name, node_key):
                self.logger.debug("cache hit - already matched")
                return True

            return False

    def check_msg_cache(self, file: str, node_key: str, msg_id: str) -> bool:
        to_hash = str(file) + str(node_key) + str(msg_id)
        event_hash = hashlib.sha256(to_hash.encode()).hexdigest()
        return bool(self.message_cache.get(event_hash))

    def update_msg_cache(self, file: str, node_key: str, msg_id: str) -> None:
        to_hash = str(file) + str(node_key) + str(msg_id)
        event_hash = hashlib.sha256(to_hash.encode()).hexdigest()
        self.message_cache.set(event_hash, "1")

    def check_hit_cache(self, file: str, node_key: str) -> bool:
        to_hash = str(file) + str(node_key)
        event_hash = hashlib.sha256(to_hash.encode()).hexdigest()
        return bool(self.hit_cache.get(event_hash))

    def update_hit_cache(self, file: str, node_key: str) -> None:
        to_hash = str(file) + str(node_key)
        event_hash = hashlib.sha256(to_hash.encode()).hexdigest()
        self.hit_cache.set(event_hash, "1")

    def lambda_handler_fn(self, events: Any, context: Any) -> None:
        # Parse sns message
        self.logger.debug(f"handling events: {events} context: {context}")

        client = GraphClient()

        s3 = get_s3_client(self.is_local)

        load_plugins(
            os.environ["BUCKET_PREFIX"], s3, os.path.abspath(MODEL_PLUGINS_DIR)
        )

        for event in events["Records"]:
            if not self.is_local:
                LOGGER.debug(f'event body: {event["body"]}')
                event = json.loads(event["body"])["Records"][0]
            data = parse_s3_event(s3, event)

            message = json.loads(data)

            LOGGER.info(f'Executing Analyzer: {message["key"]}')

            with self.metric_reporter.histogram_ctx(
                "analyzer-executor.download_s3_file"
            ):
                analyzer = download_s3_file(
                    s3,
                    f"{os.environ['BUCKET_PREFIX']}-analyzers-bucket",
                    message["key"],
                )
            analyzer_name = message["key"].split("/")[-2]

            subgraph = SubgraphView.from_proto(client, bytes(message["subgraph"]))

            # TODO: Validate signature of S3 file
            LOGGER.info(f"event {event}")
            rx: Connection
            tx: Connection
            rx, tx = Pipe(duplex=False)
            p = Process(
                target=self.execute_file,
                args=(analyzer_name, analyzer, subgraph, tx, "", self.chunk_size),
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
                    self.logger.info("execution complete")
                    break

                # emit any hits to an S3 bucket
                if isinstance(result, ExecutionHit):
                    self.logger.info(
                        f"emitting event for {analyzer_name} {result.analyzer_name} {result.root_node_key}"
                    )
                    with self.metric_reporter.histogram_ctx(
                        "analyzer-executor.emit_event.ms",
                        (TagPair("analyzer_name", result.analyzer_name),),
                    ):
                        emit_event(s3, result, self.is_local)
                    self.update_msg_cache(
                        analyzer, result.root_node_key, message["key"]
                    )
                    self.update_hit_cache(analyzer_name, result.root_node_key)

                assert not isinstance(
                    result, ExecutionFailed
                ), f"Analyzer {analyzer_name} failed."

            p.join()

    def exec_analyzers(
        self,
        dg_client,
        file: str,
        msg_id: str,
        nodes: List[BaseView],
        analyzers: Dict[str, Analyzer],
        sender: Any,
    ):
        if not analyzers:
            self.logger.warning("Received empty dict of analyzers")
            return

        if not nodes:
            self.logger.warning("Received empty array of nodes")

        for node in nodes:
            querymap: Dict[str, List[Queryable]] = defaultdict(list)

            for an_name, analyzer in analyzers.items():
                if self.check_caches(file, msg_id, node.node_key, an_name):
                    continue

                queries = analyzer.get_queries()
                if isinstance(queries, list) or isinstance(queries, tuple):
                    querymap[an_name].extend(queries)
                else:
                    querymap[an_name].append(queries)

            for an_name, queries in querymap.items():
                analyzer = analyzers[an_name]

                for query in queries:
                    # TODO: Whether it was a hit or not is a good Tag
                    tags = (TagPair("analyzer_name", an_name),)
                    with self.metric_reporter.histogram_ctx(
                        "analyzer-executor.query_first.ms", tags
                    ):
                        response = query.query_first(
                            dg_client, contains_node_key=node.node_key
                        )
                    if response:
                        self.logger.debug(
                            f"Analyzer '{an_name}' received a hit, executing on_response()"
                        )
                        with self.metric_reporter.histogram_ctx(
                            "analyzer-executor.on_response.ms", tags
                        ):
                            analyzer.on_response(response, sender)

    def execute_file(
        self, name: str, file: str, graph: SubgraphView, sender, msg_id, chunk_size
    ):
        try:
            pool = ThreadPool(processes=4)

            exec(file, globals())
            client = GraphClient()

            analyzers = get_analyzer_objects(client)
            if not analyzers:
                self.logger.warning(f"Got no analyzers for file: {name}")

            self.logger.info(f"Executing analyzers: {[an for an in analyzers.keys()]}")

            for nodes in chunker([n for n in graph.node_iter()], chunk_size):
                self.logger.info(f"Querying {len(nodes)} nodes")

                def exec_analyzer(nodes, sender):
                    try:
                        self.exec_analyzers(
                            client, file, msg_id, nodes, analyzers, sender
                        )

                        return nodes
                    except Exception as e:
                        self.logger.error(traceback.format_exc())
                        self.logger.error(
                            f"Execution of {name} failed with {e} {e.args}"
                        )
                        sender.send(ExecutionFailed())
                        raise

                # exec_analyzer(nodes, sender)
                pool.apply_async(exec_analyzer, args=(nodes, sender))

            pool.close()

            pool.join()

            sender.send(ExecutionComplete())

        except Exception as e:
            self.logger.error(traceback.format_exc())
            self.logger.error(f"Execution of {name} failed with {e} {e.args}")
            sender.send(ExecutionFailed())
            raise


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


def get_analyzer_objects(dgraph_client: GraphClient) -> Dict[str, Analyzer]:
    clsmembers = inspect.getmembers(sys.modules[__name__], inspect.isclass)
    return {
        an[0]: an[1].build(dgraph_client)
        for an in clsmembers
        if is_analyzer(an[0], an[1])
    }


def chunker(seq, size):
    return [seq[pos : pos + size] for pos in range(0, len(seq), size)]


def emit_event(s3, event: ExecutionHit, is_local: bool) -> None:
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

    if is_local:
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


def get_s3_client(is_local: bool):
    if is_local:
        return boto3.resource(
            "s3",
            endpoint_url="http://s3:9000",
            aws_access_key_id="minioadmin",
            aws_secret_access_key="minioadmin",
        )

    else:
        return boto3.resource("s3")
