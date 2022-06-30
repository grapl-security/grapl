from __future__ import annotations
from typing import (
    Dict,
    Iterator,
    MutableMapping,
)

from python_proto.api import MergedEdgeList, MergedGraph
from grapl_analyzerlib.grapl_client import GraphClient


class SubgraphView:
    def __init__(
        self, nodes: dict[str, BaseView], edges: MutableMapping[str, MergedEdgeList]
    ) -> None:
        self.nodes = nodes
        self.edges = edges

    @staticmethod
    def from_proto(graph_client: GraphClient, subgraph: MergedGraph) -> SubgraphView:
        from grapl_analyzerlib.view_from_proto import view_from_proto

        nodes = {
            k: view_from_proto(graph_client, node) for k, node in subgraph.nodes.items()
        }

        return SubgraphView(nodes, subgraph.edges)

    def node_iter(self) -> Iterator[BaseView]:
        yield from self.nodes.values()


from grapl_analyzerlib.prelude import BaseView
