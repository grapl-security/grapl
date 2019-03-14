import json
from multiprocessing import Pipe
from multiprocessing.connection import Connection
from typing import Tuple, Any, Dict

from pydgraph import DgraphClient, DgraphClientStub

from graph import Process


# Look for processes with svchost.exe in their name with non services.exe parents
def signature_graph() -> str:
    child = Process() \
        .with_image_name(contains="svchost.exe") \
        .with_node_key(eq='$a')

    parent = Process() \
        .with_image_name(contains=Not("services.exe"))
    return parent.with_child(child).to_query()


def _analyzer(client: DgraphClient, graph: Subgraph, sender: Connection):
    for node_key in graph.subgraph.nodes:
        res = client.query(signature_graph(), variables={'$a': node_key})

        if not (res and res.json):
            print('res was empty')
            continue

        res = json.loads(res.json)

        if [sender.send(make_hit(match)) for match in res.get('q0', [])]:
            print('Got a hit for {}'.format(node_key))

    sender.send(ExecutionComplete())


def make_hit(hit):
    # type: (Dict[str, Any]) -> ExecutionHit
    svc_uid = NodeRef(hit['uid'], 'Process')
    parent_uid = NodeRef(hit['~children'][0]['uid'], 'Process')

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


# class MockNode(object):
#     pass
#
#
# class MockSubgraph(object):
#     def __init__(self):
#         self.nodes = {}
#         self.edges = {}
#
# # # Testcase
# # # Given: A graph with an svchost.exe process and a malware.exe parent process
# # # When: We analyze a subgraph with the child process uid
# # # Then: We will emit an ExecutionHit
# #
# # if __name__ == '__main__':
# #     subgraph = MockSubgraph()
# #     subgraph.nodes["uuid-nodekey"] = MockNode()
# #     rx, tx = Pipe(duplex=False)  # type: Tuple[Connection, Connection]

