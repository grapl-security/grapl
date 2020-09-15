from typing import *

from pydgraph import DgraphClient

from grapl_graph_descriptions.graph_description_pb2 import EdgeList, Graph


class SubgraphView(object):
    def __init__(
        self, nodes: Dict[str, "BaseView"], edges: MutableMapping[str, EdgeList]
    ) -> None:
        self.nodes = nodes
        self.edges = edges

    @staticmethod
    def from_proto(dgraph_client: DgraphClient, s: bytes) -> "SubgraphView":
        from grapl_analyzerlib.prelude import BaseView

        subgraph = Graph()
        subgraph.ParseFromString(s)

        nodes = {
            k: BaseView.from_proto(dgraph_client, node)
            for k, node in subgraph.nodes.items()
        }
        return SubgraphView(nodes, subgraph.edges)

    def node_iter(self) -> Iterator["BaseView"]:
        for node in self.nodes.values():
            yield node

    def process_iter(self) -> Iterator["ProcessView"]:
        for node in self.nodes.values():
            maybe_node = node.into_view(ProcessView)
            if maybe_node:
                yield maybe_node

    def file_iter(self) -> Iterator["FileView"]:
        for node in self.nodes.values():
            maybe_node = node.into_view(FileView)
            if maybe_node:
                yield maybe_node


from grapl_analyzerlib.prelude import BaseView, ProcessView, FileView
