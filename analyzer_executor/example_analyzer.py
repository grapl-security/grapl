import time
import json
from multiprocessing.connection import Connection
from typing import Any, Dict

from pydgraph import DgraphClient, DgraphClientStub

from graph import Process, Not, batch_queries

# Look for processes with svchost.exe in their name with non services.exe parents
def signature_graph(node_key) -> str:
    child = Process() \
        .with_image_name(contains="svchost.exe") \
        .with_node_key(eq=node_key)

    parent = Process() \
        .with_image_name(contains=Not("services.exe"))
    return parent.with_child(child).to_query()


def _analyzer(client: DgraphClient, graph: Subgraph, sender: Connection):
    queries = [signature_graph(node_key) for node_key in graph.subgraph.nodes]
    print("Batching {} queries".format(len(graph.subgraph.nodes)))
    batched = batch_queries(queries)
    print("Querying: {}".format(int(time.time())))
    response = json.loads(client.query(batched).json)
    print("Queried: {}".format(int(time.time())))

    for hits in response.values():
        for hit in hits:
            sender.send(make_hit(hit))
            print('Got a hit for {}'.format(hit['uid']))

    sender.send(ExecutionComplete())
    print('Execution complete')


def make_hit(hit):
    # type: (Dict[str, Any]) -> ExecutionHit
    print('hit is {}'.format(hit))
    svc_uid = NodeRef(hit['children'][0]['uid'], 'Process')
    parent_uid = NodeRef(hit['uid'], 'Process')

    return ExecutionHit(
        'svchost-non-services-parent',
        [parent_uid, svc_uid],
        [(parent_uid.uid, "children", svc_uid.uid)]
    )


def analyzer(graph: Subgraph, sender: Connection):
    try:
        print('analyzing')
        client = DgraphClient(DgraphClientStub('db.mastergraph:9080'))
        _analyzer(client, graph, sender)
    except Exception as e:
        print('analyzer failed: {}'.format(e))
        sender.send(None)
