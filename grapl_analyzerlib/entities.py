from collections import defaultdict
from typing import Iterator
from typing import Optional, List, Dict, Any, Union

from pydgraph import DgraphClient

import grapl_analyzerlib.external_ip_node
import grapl_analyzerlib.node_types as node_types
import grapl_analyzerlib.outbound_connection_node as outbound_connection_node
from grapl_analyzerlib import graph_description_pb2, process_node, file_node, dynamic_node
from grapl_analyzerlib.querying import flatten_nodes


class EdgeView(object):
    def __init__(
            self, from_neighbor_key: str, to_neighbor_key: str, edge_name: str
    ) -> None:
        self.from_neighbor_key = from_neighbor_key
        self.to_neighbor_key = to_neighbor_key
        self.edge_name = edge_name


class NodeView(object):
    def __init__(self, node: Union['node_types.PV', 'node_types.FV', 'node_types.EIPV', 'node_types.OCV', 'node_types.DNV']):
        self.node = node

    @staticmethod
    def from_raw(dgraph_client: DgraphClient, node: Any) -> 'node_types.N':
        if node.HasField("process_node"):
            return NodeView(process_node.ProcessView(dgraph_client, node.process_node.node_key))
        elif node.HasField("file_node"):
            return NodeView(file_node.FileView(dgraph_client, node.file_node.node_key))
        elif node.HasField("ip_address_node"):
            return NodeView(grapl_analyzerlib.external_ip_node.ExternalIpView(dgraph_client, node.ip_address_node.node_key))
        elif node.HasField("outbound_connection_node"):
            return NodeView(outbound_connection_node.OutboundConnectionView(dgraph_client, node.outbound_connection_node.node_key))
        elif node.HasField("dynamic_node"):
            return NodeView(dynamic_node.DynamicNodeView(dgraph_client, node.dynamic_node.node_key, node.dynamic_node.node_type))
        else:
            raise Exception("Invalid Node Type")

    def as_process_view(self) -> Optional['node_types.PV']:
        if isinstance(self.node, process_node.ProcessView):
            return self.node
        return None

    def as_file_view(self) -> Optional['node_types.FV']:
        if isinstance(self.node, file_node.FileView):
            return self.node
        return None

    def as_dynamic_node(self) -> Optional['node_types.DNV']:
        if isinstance(self.node, dynamic_node.DynamicNodeView):
            return self.node
        return None

    def to_adjacency_list(self) -> Dict[str, Any]:
        all_nodes = flatten_nodes(self.node)
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
    def from_proto(dgraph_client: DgraphClient, s: bytes) -> 'node_types.S':
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

    def process_iter(self) -> Iterator['node_types.PV']:
        for node in self.nodes.values():
            maybe_node = node.as_process_view()
            if maybe_node:
                yield maybe_node

    def file_iter(self) -> Iterator['node_types.FV']:
        for node in self.nodes.values():
            maybe_node = node.as_file_view()
            if maybe_node:
                yield maybe_node


