from __future__ import annotations
from typing import (
    Dict,
    Iterator,
    MutableMapping,
)

from pydgraph import DgraphClient

from graplinc.grapl.api.graph.v1beta1.types_pb2 import MergedEdgeList, MergedGraph


class SubgraphView(object):
    def __init__(
        self, nodes: Dict[str, BaseView], edges: MutableMapping[str, EdgeList]
    ) -> None:
        self.nodes = nodes
        self.edges = edges

    @staticmethod
    def from_proto(dgraph_client: DgraphClient, s: bytes) -> SubgraphView:
        from grapl_analyzerlib.view_from_proto import view_from_proto

        subgraph = MergedGraph()
        subgraph.ParseFromString(s)

        nodes = {
            k: view_from_proto(dgraph_client, node)
            for k, node in subgraph.nodes.items()
        }
        return SubgraphView(nodes, subgraph.edges)

    def node_iter(self) -> Iterator[BaseView]:
        for node in self.nodes.values():
            yield node


from grapl_analyzerlib.prelude import BaseView
