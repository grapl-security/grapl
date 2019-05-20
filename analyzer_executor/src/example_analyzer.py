from multiprocessing.connection import Connection

import os
from pydgraph import DgraphClient, DgraphClientStub

from graph import Process, Not, batch_queries
from analyzerlib import analyze_by_signature, ExecutionComplete, NodeRef, ExecutionHit, Subgraph

# Look for processes with svchost.exe in their name with non services.exe parents
def signature_graph() -> str:
    child = Process() \
        .with_image_name(contains="svchost.exe") \

    parent = Process() \
        .with_image_name(contains=Not("services.exe"))
    return parent.with_child(child).to_query()


def _analyzer(client: DgraphClient, graph: Subgraph, sender: Connection):
    hits = analyze_by_signature(client, graph, signature_graph)

    for hit in hits:
        sender.send(ExecutionHit.from_parent_child('suspicious-svchost', hit))

    sender.send(ExecutionComplete())


def analyzer(graph: Subgraph, sender: Connection):
    try:
        print('analyzing')
        alpha_names = os.environ['MG_ALPHAS'].split(",")

        client_stubs = [DgraphClientStub('{}:9080'.format(name)) for name in alpha_names]
        client = DgraphClient(*client_stubs)
        _analyzer(client, graph, sender)
    except Exception as e:
        print('analyzer failed: {}'.format(e))
        sender.send(None)
