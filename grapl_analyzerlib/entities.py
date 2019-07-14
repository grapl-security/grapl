from collections import defaultdict
from typing import Iterator, Type
from typing import Optional, List, Dict, Any, Union

import grapl_analyzerlib.entity_queries as entity_queries
from pydgraph import DgraphClient

import grapl_analyzerlib.external_ip_node
from grapl_analyzerlib import graph_description_pb2
from grapl_analyzerlib.file_node import FileView
from grapl_analyzerlib.node_types import *
from grapl_analyzerlib.outbound_connection_node import OutboundConnectionView
from grapl_analyzerlib.dynamic_node import DynamicNodeView
from grapl_analyzerlib.process_node import ProcessView


class EdgeView(object):
    def __init__(
            self, from_neighbor_key: str, to_neighbor_key: str, edge_name: str
    ) -> None:
        self.from_neighbor_key = from_neighbor_key
        self.to_neighbor_key = to_neighbor_key
        self.edge_name = edge_name


class NodeView(object):
    def __init__(self, node: Union[PV, FV, EIPV, OCV, DNV]):
        self.node = node

    @staticmethod
    def from_raw(dgraph_client: DgraphClient, node: Any) -> N:
        if node.HasField("process_node"):
            return NodeView(ProcessView(dgraph_client, node.process_node.node_key))
        elif node.HasField("file_node"):
            return NodeView(FileView(dgraph_client, node.file_node.node_key))
        elif node.HasField("ip_address_node"):
            return NodeView(grapl_analyzerlib.external_ip_node.ExternalIpView(dgraph_client, node.ip_address_node.node_key))
        elif node.HasField("outbound_connection_node"):
            return NodeView(OutboundConnectionView(dgraph_client, node.outbound_connection_node.node_key))
        elif node.HasField("dynamic_node"):
            return NodeView(DynamicNodeView(dgraph_client, node.dynamic_node.node_key, node.dynamic_node.node_type))
        else:
            raise Exception("Invalid Node Type")

    def as_process_view(self) -> Optional[PV]:
        if isinstance(self.node, ProcessView):
            return self.node
        return None

    def as_file_view(self) -> Optional[FV]:
        if isinstance(self.node, FileView):
            return self.node
        return None

    def as_dynamic_node(self) -> Optional[DNV]:
        if isinstance(self.node, DynamicNodeView):
            return self.node
        return None

    def to_adjacency_list(self) -> Dict[str, Any]:
        all_nodes = entity_queries.flatten_nodes(self.node)
        node_dicts = defaultdict(dict)
        edges = defaultdict(list)
        for i, node in enumerate(all_nodes):
            root = False
            if i == 0:
                root = True

            node_dict = node.to_dict(root)
            node_dicts[node_dict['node']['node_key']] = node_dict['node']

            edges[node_dict['node']['node_key']].extend(node_dict['edges'])

        return {'nodes': node_dicts, 'edges': edges}


class SubgraphView(object):
    def __init__(
        self, nodes: Dict[str, NodeView], edges: Dict[str, List[EdgeView]]
    ) -> None:
        self.nodes = nodes
        self.edges = edges

    @staticmethod
    def from_proto(dgraph_client: DgraphClient, s: bytes) -> S:
        subgraph = graph_description_pb2.GraphDescription()
        subgraph.ParseFromString(s)

        nodes = {
            k: NodeView.from_raw(dgraph_client, node)
            for k, node in subgraph.nodes.items()
        }
        return SubgraphView(nodes, subgraph.edges)

    def node_iter(self) -> Iterator[NodeView]:
        for node in self.nodes.values():
            yield node

    def process_iter(self) -> Iterator[PV]:
        for node in self.nodes.values():
            maybe_node = node.as_process_view()
            if maybe_node:
                yield maybe_node

    def file_iter(self) -> Iterator[FV]:
        for node in self.nodes.values():
            maybe_node = node.as_file_view()
            if maybe_node:
                yield maybe_node


