import os

from multiprocessing.connection import Connection

from grapl_analyzerlib.entities import SubgraphView, ProcessQuery, ProcessView
from grapl_analyzerlib.entity_queries import Not
from grapl_analyzerlib.execution import ExecutionFailed, ExecutionComplete, ExecutionHit

from pydgraph import DgraphClient, DgraphClientStub


# Look for processes with svchost.exe in their name with non services.exe parents
def query(dgraph_client: DgraphClient, node_key: str) -> ProcessView:

    return (
        ProcessQuery()
            .with_process_name(contains=[Not("services.exe"), Not("lsass.exe")])
            .with_children(ProcessQuery().with_process_name(contains="svchost"))
            .query_first(dgraph_client, node_key)
    )


def _analyzer(client: DgraphClient, graph: SubgraphView, sender: Connection):
    print(f'Analyzing {len(graph.nodes)} nodes')
    for node in graph.nodes.values():
        node = node.node
        if not isinstance(node, ProcessView):
            continue
        print('Analyzing Process Node')
        response = query(client, node.node_key)
        print(response)
        if response:
            print(f"Got a response {response.node_key}")
            print(f"Debug view: {response.to_dict(root=True)}")
            sender.send(
                ExecutionHit(
                    analyzer_name="svchost_unusual_parent",
                    node_view=response,
                    risk_score=50,
                )
            )

    sender.send(ExecutionComplete())


def analyzer(client: DgraphClient, graph: SubgraphView, sender: Connection):
    try:
        _analyzer(client, graph, sender)
    except Exception as e:
        print(f"analyzer failed: {e}")
        sender.send(ExecutionFailed())
