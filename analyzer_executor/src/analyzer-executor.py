import base64
import hashlib
import inspect
import json
import os
import random
import sys
import traceback
from concurrent.futures import ThreadPoolExecutor
from multiprocessing import Process, Pipe
from multiprocessing.connection import Connection
from multiprocessing.pool import ThreadPool
from typing import Any, Optional, Tuple, List, Dict, Type, Set

import boto3
import redis
from collections import defaultdict
from grapl_analyzerlib.analyzer import Analyzer

from grapl_analyzerlib.execution import ExecutionHit, ExecutionComplete, ExecutionFailed
from grapl_analyzerlib.nodes.any_node import NodeView
from grapl_analyzerlib.nodes.queryable import Queryable, traverse_query_iter, generate_query
from grapl_analyzerlib.nodes.subgraph_view import SubgraphView
from pydgraph import DgraphClientStub, DgraphClient

IS_RETRY = os.environ['IS_RETRY']


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


def is_analyzer(analyzer_name, analyzer_cls):
    if analyzer_name == 'Analyzer':
        return False
    return hasattr(analyzer_cls, 'get_queries') and \
           hasattr(analyzer_cls, 'build') and \
           hasattr(analyzer_cls, 'on_response')


def get_analyzer_objects(dgraph_client: DgraphClient) -> Dict[str, Analyzer]:
    clsmembers = inspect.getmembers(sys.modules[__name__], inspect.isclass)
    return {an[0]: an[1].build(dgraph_client) for an in clsmembers if is_analyzer(an[0], an[1])}


def check_caches(file_hash: str, msg_id: str, node_key: str, analyzer_name: str) -> bool:
    if check_msg_cache(file_hash, node_key, msg_id):
        print('cache hit - already processed')
        return True

    if check_hit_cache(analyzer_name, node_key):
        print('cache hit - already matched')
        return True

    return False


def handle_result_graphs(analyzer, result_graphs, sender):
    print(f'Result graph: {type(analyzer)} {result_graphs[0]}')
    for result_graph in result_graphs:
        try:
            analyzer.on_response(result_graph, sender)
        except Exception as e:
            print(f'Analyzer {analyzer} failed with {e}')
            sender.send(ExecutionFailed)
            raise e


def get_analyzer_query_types(query: Queryable) -> Set[Type[Queryable]]:
    query_types = set()
    for node in traverse_query_iter(query):
        query_types.add(node.view_type)
    return query_types


def exec_analyzers(dg_client, file: str, msg_id: str, nodes: List[NodeView], analyzers: Dict[str, Analyzer], sender: Any):
    if not analyzers:
        print('Received empty dict of analyzers')
        return

    if not nodes:
        print("Received empty array of nodes")

    result_name_to_analyzer = {}
    query_str = ""

    for node in nodes:
        querymap = defaultdict(list)

        for an_name, analyzer in analyzers.items():
            if check_caches(file, msg_id, node.node_key, an_name):
                continue

            analyzer = analyzer  # type: Analyzer
            queries = analyzer.get_queries()
            if isinstance(queries, list) or isinstance(queries, tuple):

                querymap[an_name].extend(queries)
            else:
                querymap[an_name].append(queries)

        for an_name, queries in querymap.items():
            analyzer = analyzers[an_name]

            for i, query in enumerate(queries):
                analyzer_query_types = get_analyzer_query_types(query)
                if type(node.node) not in analyzer_query_types:
                    continue
                r = str(random.randint(10, 100))
                result_name = f'{an_name}u{int(node.uid, 16)}i{i}r{r}'.strip().lower()
                result_name_to_analyzer[result_name] = (an_name, analyzer, query.view_type)
                query_str += '\n'
                query_str += generate_query(
                    query_name=result_name,
                    binding_modifier=result_name,
                    root=query,
                    contains_node_key=node.node_key,
                )

    if not query_str:
        print('No nodes to query')
        return

    txn = dg_client.txn(read_only=True)
    try:
        response = json.loads(txn.query(query_str).json)
    finally:
        txn.discard()

    print(f'query to analyzer map {result_name_to_analyzer}')

    analyzer_to_results = defaultdict(list)
    for result_name, results in response.items():
        for result in results:
            analyzer_meta = result_name_to_analyzer[result_name]  # type: Tuple[str, Analyzer, Type[Viewable]]
            an_name, analyzer, view_type = analyzer_meta[0], analyzer_meta[1], analyzer_meta[2]

            result_graph = view_type.from_dict(dg_client, result)

            # next(inspect.getfullargspec(analyzer.on_response).annotations.values().__iter__())
            response_ty = inspect.getfullargspec(analyzer.on_response).annotations.get('response')

            if response_ty == NodeView:
                print('Analyzer on_response is expecting a NodeView')
                result_graph = NodeView.from_view(result_graph)

            analyzer_to_results[an_name].append(result_graph)

    with ThreadPoolExecutor(max_workers=6) as executor:

        for an_name, result_graphs in analyzer_to_results.items():
            analyzer = analyzers[an_name]
            executor.submit(handle_result_graphs, analyzer, result_graphs, sender)
        executor.shutdown(wait=True)


def chunker(seq, size):
    return [seq[pos:pos + size] for pos in range(0, len(seq), size)]


def execute_file(name: str, file: str, graph: SubgraphView, sender, msg_id):
    alpha_names = os.environ["MG_ALPHAS"].split(",")

    try:
        pool = ThreadPool(processes=4)

        exec(file, globals())
        client_stubs = [DgraphClientStub(f"{a_name}:9080") for a_name in alpha_names]
        client = DgraphClient(*client_stubs)

        analyzers = get_analyzer_objects(client)
        if not analyzers:
            print(f'Got no analyzers for file: {name}')

        print(f'Executing analyzers: {[an for an in analyzers.keys()]}')

        chunk_size = 100

        if IS_RETRY == "True":
            chunk_size = 10

        for nodes in chunker([n for n in graph.node_iter()], chunk_size):
            print(f'Querying {len(nodes)} nodes')

            def exec_analyzer(nodes, sender):
                try:
                    exec_analyzers(client, file, msg_id, nodes, analyzers, sender)

                    return nodes
                except Exception as e:
                    print(traceback.format_exc())
                    print(f'Execution of {name} failed with {e} {e.args}')
                    sender.send(ExecutionFailed())
                    raise

            exec_analyzer(nodes, sender)
            pool.apply_async(exec_analyzer, args=(nodes, sender))

        pool.close()

        pool.join()

        sender.send(ExecutionComplete())

    except Exception as e:
        print(traceback.format_exc())
        print(f'Execution of {name} failed with {e} {e.args}')
        sender.send(ExecutionFailed())
        raise


def emit_event(event: ExecutionHit) -> None:
    print(f"emitting event for: {event.analyzer_name, event.nodes}")

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
MESSAGECACHE_PORT = int(os.environ['MESSAGECACHE_PORT'])

message_cache = redis.Redis(host=MESSAGECACHE_ADDR, port=MESSAGECACHE_PORT, db=0)

HITCACHE_ADDR = os.environ['HITCACHE_ADDR']
HITCACHE_PORT = os.environ['HITCACHE_PORT']

hit_cache = redis.Redis(host=HITCACHE_ADDR, port=int(HITCACHE_PORT), db=0)


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

        print(f'Executing Analyzer: {message["key"]}')
        analyzer = download_s3_file(f"{os.environ['BUCKET_PREFIX']}-analyzers-bucket", message["key"])
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
                print(f"emitting event for {analyzer_name} {result.analyzer_name} {result.root_node_key}")
                emit_event(result)
                update_msg_cache(analyzer, result.root_node_key, message['key'])
                update_hit_cache(analyzer_name, result.root_node_key)

            assert not isinstance(
                result, ExecutionFailed
            ), f"Analyzer {analyzer_name} failed."

        p.join()
