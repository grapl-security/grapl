from __future__ import annotations
from typing import (
    Dict,
    Iterator,
    MutableMapping,
)

from python_proto.api import MergedEdgeList, MergedGraph
from graplinc.grapl.api.graph.v1beta1.types_pb2 import MergedEdgeList, MergedGraph
from grapl_analyzerlib.grapl_client import GraphClient


class SubgraphView(object):
    def __init__(
        self, nodes: Dict[str, BaseView], edges: MutableMapping[str, MergedEdgeList]
    ) -> None:
        self.nodes = nodes
        self.edges = edges

    @staticmethod
    def from_proto(graph_client: GraphClient, s: bytes) -> SubgraphView:
        from grapl_analyzerlib.view_from_proto import view_from_proto

        subgraph = MergedGraph()
        subgraph.ParseFromString(s)

        nodes = {
            k: view_from_proto(graph_client, node) for k, node in subgraph.nodes.items()
        }
        return SubgraphView(nodes, subgraph.edges)

    def node_iter(self) -> Iterator[BaseView]:
        for node in self.nodes.values():
            yield node


from grapl_analyzerlib.prelude import BaseView
