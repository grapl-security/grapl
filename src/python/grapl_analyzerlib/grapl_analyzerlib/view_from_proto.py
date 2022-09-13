from grapl_analyzerlib.grapl_client import GraphClient
from grapl_analyzerlib.prelude import (
    BaseView,
)
from python_proto.api import MergedNode


def view_from_proto(graph_client: GraphClient, node: MergedNode) -> BaseView:
    return BaseView(
        node.uid,
        node.node_key,
        graph_client,
        node_types={node.node_type},
    )
