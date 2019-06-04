from multiprocessing.connection import Connection

import os
from pydgraph import DgraphClient, DgraphClientStub

from grapl_analyzerlib.entity_queries import Not
from grapl_analyzerlib.entities import ProcessQuery, SubgraphView
from grapl_analyzerlib.execution import ExecutionComplete, ExecutionFailed

# Look for processes with svchost.exe in their name with non services.exe parents
def query(dgraph_client: DgraphClient, node_key: str) -> ProcessQuery:
    child = ProcessQuery().with_process_name(contains="svchost.exe")

    parent = ProcessQuery().with_process_name(contains=Not("services.exe"))

    return parent.with_children(child).query_first(dgraph_client, node_key)


def _analyzer(client: DgraphClient, graph: SubgraphView, sender: Connection):

    for node in graph.process_iter():
        response = query(client, node.node_key)
        if response:
            print("Got a response")

    sender.send(ExecutionComplete())


def analyzer(graph: SubgraphView, sender: Connection):
    try:
        print("analyzing")
        alpha_names = os.environ["MG_ALPHAS"].split(",")

        client_stubs = [
            DgraphClientStub("{}:9080".format(name)) for name in alpha_names
        ]
        client = DgraphClient(*client_stubs)
        _analyzer(client, graph, sender)
    except Exception as e:
        print("analyzer failed: {}".format(e))
        sender.send(ExecutionFailed())
