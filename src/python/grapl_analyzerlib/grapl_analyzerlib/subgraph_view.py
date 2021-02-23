from __future__ import annotations
from typing import (
    Dict,
    Iterator,
    MutableMapping,
)

from grapl_analyzerlib.grapl_client import GraphClient
from graplinc.grapl.api.graph.v1beta1.types_pb2 import EdgeList, Graph


class SubgraphView(object):
    def __init__(
        self, nodes: Dict[str, BaseView], edges: MutableMapping[str, EdgeList]
    ) -> None:
        self.nodes = nodes
        self.edges = edges

    @staticmethod
    def from_proto(graph_client: GraphClient, s: bytes) -> SubgraphView:
        from grapl_analyzerlib.view_from_proto import view_from_proto

        subgraph = Graph()
        subgraph.ParseFromString(s)

        nodes = {
            k: view_from_proto(graph_client, node)
            for k, node in subgraph.nodes.items()
        }
        return SubgraphView(nodes, subgraph.edges)

    def node_iter(self) -> Iterator[BaseView]:
        for node in self.nodes.values():
            yield node

    def process_iter(self) -> Iterator[ProcessView]:
        for node in self.nodes.values():
            maybe_node = node.into_view(ProcessView)
            if maybe_node:
                yield maybe_node

    def file_iter(self) -> Iterator[FileView]:
        for node in self.nodes.values():
            maybe_node = node.into_view(FileView)
            if maybe_node:
                yield maybe_node


from grapl_analyzerlib.prelude import BaseView, ProcessView, FileView
