from __future__ import annotations

import base64
import hashlib
import inspect
import json
import os
import sys
import traceback
from collections import defaultdict
from datetime import datetime
from logging import Logger
from multiprocessing import Pipe, Process
from multiprocessing.connection import Connection
from multiprocessing.pool import ThreadPool
from pathlib import Path
from typing import TYPE_CHECKING, Any, Dict, Iterable, Iterator, List, Optional, Union

import boto3  # type: ignore
import redis
from analyzer_executor_lib.sqs_types import S3PutRecordDict, SQSMessageBody
from grapl_analyzerlib.analyzer import Analyzer
from grapl_analyzerlib.execution import ExecutionComplete, ExecutionFailed, ExecutionHit
from grapl_analyzerlib.grapl_client import GraphClient
from grapl_analyzerlib.nodes.base import BaseView
from grapl_analyzerlib.plugin_retriever import load_plugins
from grapl_analyzerlib.queryable import Queryable
from grapl_analyzerlib.subgraph_view import SubgraphView
from grapl_common.env_helpers import S3ResourceFactory, SQSClientFactory
from grapl_common.grapl_logger import get_module_grapl_logger
from grapl_common.metrics.metric_reporter import MetricReporter, TagPair

if TYPE_CHECKING:
    from mypy_boto3_s3 import S3Client, S3ServiceResource
    from mypy_boto3_sqs import SQSClient


# Set up logger (this is for the whole file, including static methods)
LOGGER = get_module_grapl_logger()

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
    def set(self, key: str, value: str) -> None:
        pass

    def get(self, key: str) -> bool:
        return False

    def delete(self, key: str) -> None:
        pass


EitherCache = Union[NopCache, redis.Redis]


class AnalyzerExecutor:

    # constants
    CHUNK_SIZE_RETRY: int = 10
    CHUNK_SIZE_DEFAULT: int = 100

    # singleton
    _singleton = None

    def __init__(
        self,
        message_cache: EitherCache,
        hit_cache: EitherCache,
        chunk_size: int,
        is_local: bool,
        logger: Logger,
        metric_reporter: MetricReporter,
    ) -> None:
        self.message_cache = message_cache
        self.hit_cache = hit_cache
        self.chunk_size = chunk_size
        self.is_local = is_local
        self.logger = logger
        self.metric_reporter = metric_reporter

    @classmethod
    def singleton(cls) -> AnalyzerExecutor:
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
            messagecache_port: Optional[int] = None
            messagecache_port_str = os.getenv("MESSAGECACHE_PORT")
            if messagecache_port_str:
                try:
                    messagecache_port = int(messagecache_port_str)
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
            hitcache_port: Optional[int] = None
            hitcache_port_str = os.getenv("HITCACHE_PORT")
            if hitcache_port_str:
                try:
                    hitcache_port = int(hitcache_port_str)
                except (TypeError, ValueError) as ex:
                    LOGGER.error(
                        f"can't connect to redis, HITCACHE_PORT couldn't cast to int"
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

    def to_event_hash(self, components: Iterable[str]) -> str:
        joined = ",".join(components)
        event_hash = hashlib.sha256(joined.encode()).hexdigest()
        return event_hash

    def check_msg_cache(self, file: str, node_key: str, msg_id: str) -> bool:
        event_hash = self.to_event_hash((file, node_key, msg_id))
        return bool(self.message_cache.get(event_hash))

    def update_msg_cache(self, file: str, node_key: str, msg_id: str) -> None:
        event_hash = self.to_event_hash((file, node_key, msg_id))
        self.message_cache.set(event_hash, "1")

    def delete_msg_cache(self, file: str, node_key: str, msg_id: str) -> None:
        """
        Only use case right now is cleaning up Redis at test time
        """
        event_hash = self.to_event_hash((file, node_key, msg_id))
        self.message_cache.delete(event_hash)

    def check_hit_cache(self, file: str, node_key: str) -> bool:
        event_hash = self.to_event_hash((file, node_key))
        return bool(self.hit_cache.get(event_hash))

    def update_hit_cache(self, file: str, node_key: str) -> None:
        event_hash = self.to_event_hash((file, node_key))
        self.hit_cache.set(event_hash, "1")

    def delete_hit_cache(self, file: str, node_key: str) -> None:
        """
        Only use case right now is cleaning up Redis at test time
        """
        event_hash = self.to_event_hash((file, node_key))
        self.hit_cache.delete(event_hash)

    async def handle_events(self, events: SQSMessageBody, context: Any) -> None:
        # Parse sns message
        self.logger.debug(f"handling events: {events} context: {context}")

        client = GraphClient()

        s3 = S3ResourceFactory(boto3).from_env()

        load_plugins(
            os.environ["DEPLOYMENT_NAME"],
            s3.meta.client,
            os.path.abspath(MODEL_PLUGINS_DIR),
        )

        for event in events["Records"]:
            data = parse_s3_event(s3, event)

            message = json.loads(data)

            LOGGER.info(f'Executing Analyzer: {message["key"]}')

            with self.metric_reporter.histogram_ctx(
                "analyzer-executor.download_s3_file"
            ):
                analyzer = download_s3_file(
                    s3,
                    f"{os.environ['DEPLOYMENT_NAME']}-analyzers-bucket",
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

            for exec_hit in self.poll_process(rx=rx, analyzer_name=analyzer_name):
                with self.metric_reporter.histogram_ctx(
                    "analyzer-executor.emit_event.ms",
                    (TagPair("analyzer_name", exec_hit.analyzer_name),),
                ):
                    emit_event(s3, exec_hit, self.is_local)
                self.update_msg_cache(analyzer, exec_hit.root_node_key, message["key"])
                self.update_hit_cache(analyzer_name, exec_hit.root_node_key)

            p.join()

    def poll_process(
        self,
        rx: Connection,
        analyzer_name: str,
    ) -> Iterator[ExecutionHit]:
        """
        Keep polling the spawned Process, and yield any ExecutionHits.
        (This will probably disappear if Analyzers move to Docker images.)
        """
        t = 0

        while True:
            p_res = rx.poll(timeout=5)
            if not p_res:
                t += 1
                LOGGER.info(
                    f"Analyzer {analyzer_name} polled for for {t * 5} seconds without result"
                )
                continue

            result: Optional[Any] = rx.recv()
            if isinstance(result, ExecutionComplete):
                self.logger.info(f"Analyzer {analyzer_name} execution complete")
                return

            # emit any hits to an S3 bucket
            if isinstance(result, ExecutionHit):
                self.logger.info(
                    f"Analyzer {analyzer_name} emitting event for:"
                    f"{result.analyzer_name} {result.root_node_key}"
                )
                yield result

            assert not isinstance(
                result, ExecutionFailed
            ), f"Analyzer {analyzer_name} failed."

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

                pool.apply_async(exec_analyzer, args=(nodes, sender))

            pool.close()

            pool.join()

            sender.send(ExecutionComplete())

        except Exception as e:
            self.logger.error(traceback.format_exc())
            self.logger.error(f"Execution of {name} failed with {e} {e.args}")
            sender.send(ExecutionFailed())
            raise


def parse_s3_event(s3: S3ServiceResource, event: S3PutRecordDict) -> str:
    try:
        bucket = event["s3"]["bucket"]["name"]
        key = event["s3"]["object"]["key"]
    except KeyError:
        LOGGER.error("Could not parse s3 event: {}", exc_info=True)
        raise
    return download_s3_file(s3, bucket, key)


def download_s3_file(s3: S3ServiceResource, bucket: str, key: str) -> str:
    obj = s3.Object(bucket, key)
    return obj.get()["Body"].read().decode("utf-8")


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


def emit_event(s3: S3ServiceResource, event: ExecutionHit, is_local: bool) -> None:
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
        f"{os.environ['DEPLOYMENT_NAME']}-analyzer-matched-subgraphs-bucket", key
    )
    obj.put(Body=event_s.encode("utf-8"))

    # TODO fargate: always emit manual events
    if is_local:
        # Local = manual eventing

        deployment_name = os.environ["DEPLOYMENT_NAME"]

        sqs = SQSClientFactory(boto3).from_env()
        send_s3_event(
            sqs,
            f"{os.environ['SQS_ENDPOINT']}/queue/{deployment_name}-engagement-creator-queue",
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
    sqs_client: SQSClient,
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
