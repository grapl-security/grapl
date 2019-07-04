import base64
import hashlib
import json
import os
import traceback

from multiprocessing import Process, Pipe
from multiprocessing.connection import Connection
from multiprocessing.pool import ThreadPool

from typing import Any, Optional, Tuple

import boto3
import redis
from botocore.exceptions import ClientError

from grapl_analyzerlib.entities import SubgraphView, ProcessView, FileView
from grapl_analyzerlib.execution import ExecutionHit, ExecutionComplete, ExecutionFailed
from pydgraph import DgraphClientStub, DgraphClient


def parse_s3_event(event) -> str:
    # Retrieve body of sns message
    # Decode json body of sns message
    print("event is {}".format(event))
    msg = json.loads(event["body"])["Message"]
    msg = json.loads(msg)

    record = msg["Records"][0]

    bucket = record["s3"]["bucket"]["name"]
    key = record["s3"]["object"]["key"]
    return download_s3_file(bucket, key)


def download_s3_file(bucket, key) -> str:
    s3 = boto3.resource("s3")
    obj = s3.Object(bucket, key)
    return obj.get()["Body"].read()


def execute_file(name: str, file: str, graph: SubgraphView, sender, msg_id):
    alpha_names = os.environ["MG_ALPHAS"].split(",")

    client_stubs = [DgraphClientStub(f"{name}:9080") for name in alpha_names]
    client = DgraphClient(*client_stubs)

    exec(file, globals())
    try:
        pool = ThreadPool(processes=64)
        results = []
        for node in graph.node_iter():
            if not node.node.node_key:
                print(f'missing key {vars(node.node.node)} type: {type(node.node)}')
                continue

            if check_msg_cache(file, node.node.node_key, msg_id):
                print('cache hit - already processed')
                continue

            if check_hit_cache(name, node.node.node_key):
                print('cache hit - already matched')
                continue

            def exec_analyzer(analyzer, client, node, sender):
                analyzer(client, node, sender)
                return node

            t = pool.apply_async(exec_analyzer, (analyzer, client, node, sender))
            results.append(t)

        for result in results:
            node = result.get()
            update_msg_cache(file, node.node.node_key, msg_id)

        sender.send(ExecutionComplete())

    except Exception as e:
        print(traceback.format_exc())
        print(f'Execution of {name} failed with {e} {e.args}')
        sender.send(ExecutionFailed())


def emit_event(event: ExecutionHit) -> None:
    print("emitting event")

    event_s = json.dumps(
        {
            "nodes": json.loads(event.nodes),
            "edges": json.loads(event.edges),
            "analyzer_name": event.analyzer_name,
            "risk_score": event.risk_score,
        }
    )
    event_hash = hashlib.sha256(event_s.encode())
    key = base64.urlsafe_b64encode(event_hash.digest()).decode("utf-8")

    s3 = boto3.resource("s3")
    obj = s3.Object("grapl-analyzer-matched-subgraphs-bucket", key)
    obj.put(Body=event_s)

    # try:
    #     obj.load()
    # except ClientError as e:
    #     if e.response['Error']['Code'] == "404":
    #     else:
    #         raise


MESSAGECACHE_ADDR = os.environ['MESSAGECACHE_ADDR']
MESSAGECACHE_PORT = os.environ['MESSAGECACHE_PORT']

message_cache = redis.Redis(host=MESSAGECACHE_ADDR, port=MESSAGECACHE_PORT, db=0)

HITCACHE_ADDR = os.environ['HITCACHE_ADDR']
HITCACHE_PORT = os.environ['HITCACHE_PORT']

hit_cache = redis.Redis(host=HITCACHE_ADDR, port=HITCACHE_PORT, db=0)


def check_msg_cache(file: str, node_key: str, msg_id: str) -> bool:
    to_hash = str(file) + str(node_key) + str(msg_id)
    event_hash = hashlib.sha256(to_hash.encode()).hexdigest()
    return bool(message_cache.get(event_hash))


def update_msg_cache(file: str, node_key: str, msg_id: str):
    to_hash = str(file) + str(node_key) + str(msg_id)
    event_hash = hashlib.sha256(to_hash.encode()).hexdigest()
    message_cache.set(event_hash, "1")


def check_hit_cache(file: str, node_key: str) -> bool:
    to_hash = str(file) + str(node_key)
    event_hash = hashlib.sha256(to_hash.encode()).hexdigest()
    return bool(hit_cache.get(event_hash))


def update_hit_cache(file: str, node_key: str):
    to_hash = str(file) + str(node_key)
    event_hash = hashlib.sha256(to_hash.encode()).hexdigest()
    hit_cache.set(event_hash, "1")


def lambda_handler(events: Any, context: Any) -> None:
    # Parse sns message
    print("handling")
    print(events)
    print(context)

    alpha_names = os.environ["MG_ALPHAS"].split(",")

    client_stubs = [DgraphClientStub("{}:9080".format(name)) for name in alpha_names]
    client = DgraphClient(*client_stubs)

    for event in events["Records"]:
        data = parse_s3_event(event)

        message = json.loads(data)

        # TODO: Use env variable for s3 bucket
        print(f'Executing Analyzer: {message["key"]}')
        analyzer = download_s3_file("grapl-analyzers-bucket", message["key"])
        analyzer_name = message["key"].split("/")[-2]
        subgraph = SubgraphView.from_proto(client, bytes(message["subgraph"]))
        
        # TODO: Validate signature of S3 file
        rx, tx = Pipe(duplex=False)  # type: Tuple[Connection, Connection]
        p = Process(target=execute_file, args=(analyzer_name, analyzer, subgraph, tx, event['messageId']))

        p.start()
        t = 0
        while True:
            p_res = rx.poll(timeout=5)
            if not p_res:
                t += 1
                print(f"Polled {analyzer_name} for {t * 5} seconds without result")
                continue
            result = rx.recv()  # type: Optional[Any]

            if isinstance(result, ExecutionComplete):
                print("execution complete")
                break

            # emit any hits to an S3 bucket
            if isinstance(result, ExecutionHit):
                print(f"emitting event for {analyzer_name} {result.root_node_key}")
                emit_event(result)
                update_msg_cache(analyzer, result.root_node_key, message['key'])
                update_hit_cache(analyzer_name, result.root_node_key)

            assert not isinstance(
                result, ExecutionFailed
            ), f"Analyzer {analyzer_name} failed."

        p.join()


