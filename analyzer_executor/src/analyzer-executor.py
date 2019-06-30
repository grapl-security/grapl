import base64
import hashlib
import json
import os
import traceback

from multiprocessing import Process, Pipe
from multiprocessing.connection import Connection
from typing import Any, Optional, Tuple

import boto3
import redis
from botocore.exceptions import ClientError

from grapl_analyzerlib.entities import SubgraphView
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

    client_stubs = [DgraphClientStub("{}:9080".format(name)) for name in alpha_names]
    client = DgraphClient(*client_stubs)

    exec(file, globals())
    try:
        from multiprocessing.pool import ThreadPool
        with ThreadPool(processes=64) as pool:
            results = []
            for node in graph.node_iter():
                if check_cache(file, node.node.node_key, msg_id):
                    print('cache hit')
                    continue

                t = pool.apply_async(analyzer, (client, node, sender))
                results.append(t)
                # TODO: Check node + analyzer file hash in redis cache, avoid reprocess of hits
            for result in results:
                result.get()
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

# COUNTCACHE_ADDR = os.environ['COUNTCACHE_ADDR']
# COUNTCACHE_PORT = os.environ['COUNTCACHE_PORT']
#
# cache = redis.Redis(host=COUNTCACHE_ADDR, port=COUNTCACHE_PORT, db=0)
cache = {}

def check_cache(file: str, node_key: str, msg_id: str) -> bool:
    to_hash = str(file) + str(node_key) + str(msg_id)
    event_hash = hashlib.sha256(to_hash.encode())
    if cache.get(event_hash):
        return True
    else:
        return False


def update_cache(file: str, node_key: str, msg_id: str):
    to_hash = str(file) + str(node_key) + str(msg_id)
    event_hash = hashlib.sha256(to_hash.encode())
    # cache.set(event_hash, True)
    cache[event_hash] = True

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
        print("creating queue")
        rx, tx = Pipe(duplex=False)  # type: Tuple[Connection, Connection]
        print("creating process")
        p = Process(target=execute_file, args=(analyzer_name, analyzer, subgraph, tx, event['messageId']))
        print("running process")
        p.start()

        while True:
            print("waiting for results")
            p_res = rx.poll(timeout=5)
            if not p_res:
                print("Polled for 5 seconds without result")
                continue
            result = rx.recv()  # type: Optional[Any]

            if isinstance(result, ExecutionComplete):
                print("execution complete")
                for node in subgraph.node_iter():
                    update_cache(analyzer, node.node.node_key, event['messageId'])

            # emit any hits to an S3 bucket
            if isinstance(result, ExecutionHit):
                print(f"emitting event for {analyzer_name}")
                emit_event(result)
                update_cache(analyzer, result.root_node_key, event['messageId'])

            assert not isinstance(
                result, ExecutionFailed
            ), "Analyzer failed."

        p.join()


