import json
from collections import defaultdict
from typing import (
    Optional,
    TypeVar,
    Tuple,
    Mapping,
    Any,
    Union,
    Type,
    Dict,
    List,
    Set,
    cast,
)

from pydgraph import DgraphClient

from grapl_analyzerlib.graph_description_pb2 import Node
from grapl_analyzerlib.nodes.asset_node import AssetView
from grapl_analyzerlib.nodes.ip_address_node import IpAddressView
from grapl_analyzerlib.nodes.ip_connection_node import IpConnectionView
from grapl_analyzerlib.nodes.ip_port_node import IpPortView
from grapl_analyzerlib.nodes.network_connection_node import NetworkConnectionView
from grapl_analyzerlib.nodes.process_inbound_network_connection import (
    ProcessInboundConnectionView,
)
from grapl_analyzerlib.nodes.process_outbound_network_connection import (
    ProcessOutboundConnectionView,
)
from grapl_analyzerlib.nodes.queryable import Queryable
from grapl_analyzerlib.nodes.viewable import Viewable

# noinspection Mypy

T = TypeVar("T")

# Proto nodes don't contain a uid so we have to fetch them. It may make sense to store these uids
# alongside the proto in the future. This makes constructing from proto relatively expensive.
def get_uid(client: DgraphClient, node_key: str) -> str:
    txn = client.txn(read_only=True)
    try:
        query = """
            query res($a: string)
            {
              res(func: eq(node_key, $a), first: 1) @cascade
               {
                 uid,
               }
             }"""
        res = txn.query(query, variables={"$a": node_key})
        res = json.loads(res.json)

        if isinstance(res["res"], list):
            if res["res"]:
                return str(res["res"][0]["uid"])
            else:
                raise Exception(f"get_uid failed for node_key: {node_key} {res}")
        else:
            return str(res["res"]["uid"])

    finally:
        txn.discard()


def raw_node_from_uid(
    dgraph_client: DgraphClient, uid: str
) -> Optional[Dict[str, Any]]:
    query = f"""
        {{
            res(func: uid("{uid}"), first: 1) {{
                uid,
                expand(_all_),
                node_type: dgraph.type
            }}
        }}
        """

    txn = dgraph_client.txn(read_only=True, best_effort=False)
    try:
        res = json.loads(txn.query(query).json)["res"]
    finally:
        txn.discard()
    if not res:
        return None
    else:
        if isinstance(res, list):
            node_type = res[0].get("node_type")
            if node_type:
                res[0]["node_type"] = res[0]["node_type"][0]
            else:
                print(f"WARN: node_type missing from {uid} {res}")

            return cast(Dict[str, Any], res[0])
        else:
            node_type = res.get("node_type")
            if node_type:
                res["node_type"] = res["node_type"][0]
            else:
                print(f"WARN: node_type missing from {uid} {res}")
            return cast(Dict[str, Any], res)


def raw_node_from_node_key(
    dgraph_client: DgraphClient, node_key: str
) -> Optional[Dict[str, Any]]:
    query = f"""
        {{
            res(func: eq(node_key, "{node_key}"), first: 1) {{
                uid,
                expand(_all_),
                node_type: dgraph.type
            }}
        }}
        """

    txn = dgraph_client.txn(read_only=True, best_effort=False)
    try:
        res = json.loads(txn.query(query).json)["res"]
    finally:
        txn.discard()
    if not res:
        return None

    try:
        if isinstance(res, list):
            node_type = res[0].get("node_type")
            if node_type:
                res[0]["node_type"] = res[0]["node_type"][0]
            else:
                print(f"WARN: node_type missing from {node_key} {res}")

            return cast(Dict[str, Any], res[0])
        else:
            node_type = res.get("node_type")
            if node_type:
                res["node_type"] = res["node_type"][0]
            else:
                print(f"WARN: node_type missing from {node_key} {res}")

            return cast(Dict[str, Any], res)
    except Exception as e:
        print(f"WARN: raw_node_from_node_key {node_key} {res} {e}")
        raise e


def flatten_nodes(root: Viewable) -> List[Viewable]:
    node_list = [root]
    already_visited = set()  # type: Set[Any]
    to_visit = [root]

    while True:
        if not to_visit:
            break

        next_node = to_visit.pop()

        if next_node in already_visited:
            continue

        neighbors = next_node.get_edges()

        for _neighbor in neighbors.values():
            if not isinstance(_neighbor, Viewable):
                neighbor = _neighbor[0]
            else:
                neighbor = _neighbor

            if isinstance(neighbor, list):
                node_list.extend(neighbor)
                to_visit.extend(neighbor)
            else:
                node_list.append(neighbor)
                to_visit.append(neighbor)

        already_visited.add(next_node)

    # Maintaining order is a convenience
    return list(dict.fromkeys(node_list))


class NodeQuery(Queryable):
    def __init__(self) -> None:
        super(NodeQuery, self).__init__(NodeView)

    def _get_unique_predicate(self) -> "Optional[Tuple[str, PropertyT]]":
        return None

    def _get_node_type_name(self) -> str:
        return None

    def _get_property_filters(self) -> Mapping[str, "PropertyFilter[Property]"]:
        return self.dynamic_property_filters

    def _get_forward_edges(self) -> Mapping[str, "Queryable"]:
        return self.dynamic_forward_edge_filters

    def _get_reverse_edges(self) -> Mapping[str, Tuple["Queryable", str]]:
        return self.dynamic_reverse_edge_filters

    def query(
        self,
        dgraph_client: DgraphClient,
        contains_node_key: Optional[str] = None,
        first: Optional[int] = 1000,
    ) -> List["NodeView"]:
        res = self.query(dgraph_client, contains_node_key, first)

        if not res:
            return []

        assert isinstance(res[0], NodeView)
        return cast("List[NodeView]", res)

    def query_first(
        self, dgraph_client: DgraphClient, contains_node_key: Optional[str] = None
    ) -> Optional["NodeView"]:
        res = self.query_first(dgraph_client, contains_node_key)
        assert (res is None) or isinstance(res, NodeView)
        return res


class NodeView(Viewable):

    def __init__(
        self, dgraph_client: DgraphClient, node_key: str, uid: str, node: Viewable
    ):
        super(NodeView, self).__init__(
            dgraph_client=dgraph_client, node_key=node_key, uid=uid
        )
        self.node = node

    def get_node_type(self) -> str:
        return self.node.get_node_type()

    @staticmethod
    def from_view(v: Viewable):
        if isinstance(v, NodeView):
            return v
        try:
            return NodeView(v.dgraph_client, v.node_key, v.uid, v)
        except Exception as e:
            print(f"ERROR from_view failed for : {v}")
            raise e

    @staticmethod
    def from_node_key(client, node_key):
        res = raw_node_from_node_key(client, node_key)
        if not res:
            return None
        else:
            try:
                return NodeView.from_dict(client, res)
            except Exception as e:
                print(f"ERROR from_node_key failed: {node_key} {res}")
                raise e

    @staticmethod
    def from_uid(client, uid):
        res = raw_node_from_uid(client, uid)
        if not res:
            return None
        else:
            try:
                return NodeView.from_dict(client, res)
            except Exception as e:
                print(f"ERROR from_node_uid failed: {uid} {res}")
                raise e

    @classmethod
    def from_dict(
        cls: Type["Viewable"], dgraph_client: DgraphClient, d: Dict[str, Any]
    ) -> "NodeView":

        node_type = d.get("node_type", d.get("dgraph.type", ""))  # type: Optional[str]
        if isinstance(node_type, list):
            node_type = node_type[0]

        _d = None
        uid = d.get("uid")  # type: Optional[str]
        if uid:
            _d = raw_node_from_uid(dgraph_client, uid)

        if _d:
            d = {**d, **_d}

        if d.get("process_id", d.get("process_name")) or node_type == "Process":
            # Type is Any but we assert the type below
            node = ProcessView.from_dict(dgraph_client, d)  # type: Any
        elif d.get("file_path") or node_type == "File":
            node = FileView.from_dict(dgraph_client, d)
        elif node_type == "IpAddress":
            node = IpAddressView.from_dict(dgraph_client, d)
        elif node_type == "IpPort":
            node = IpPortView.from_dict(dgraph_client, d)
        elif node_type == "ProcessOutboundConnection":
            node = ProcessOutboundConnectionView.from_dict(dgraph_client, d)
        elif node_type == "ProcessInboundConnection":
            node = ProcessInboundConnectionView.from_dict(dgraph_client, d)
        elif node_type == "IpConnection":
            node = IpConnectionView.from_dict(dgraph_client, d)
        elif node_type == "NetworkConnection":
            node = NetworkConnectionView.from_dict(dgraph_client, d)
        elif node_type:
            node = DynamicNodeView.from_dict(dgraph_client, d)
        else:
            raise Exception(f"Invalid scoped node type: {d}")

        assert (
            isinstance(node, ProcessView)
            or isinstance(node, FileView)
            or isinstance(node, IpAddressView)
            or isinstance(node, DynamicNodeView)
        )

        return NodeView(
            dgraph_client=dgraph_client, node_key=node.node_key, uid=node.uid, node=node
        )

    @staticmethod
    def from_proto(dgraph_client: DgraphClient, node: Node) -> "NodeView":

        if node.HasField("process_node"):
            uid = get_uid(dgraph_client, node.process_node.node_key)
            assert uid

            return NodeView(
                dgraph_client,
                node.process_node.node_key,
                uid,
                ProcessView(
                    dgraph_client=dgraph_client,
                    uid=uid,
                    node_key=node.process_node.node_key,
                    process_id=node.process_node.process_id,
                ),
            )
        elif node.HasField("file_node"):
            uid = get_uid(dgraph_client, node.file_node.node_key)

            return NodeView(
                dgraph_client,
                node.file_node.node_key,
                uid,
                FileView(
                    dgraph_client=dgraph_client,
                    uid=uid,
                    node_key=node.file_node.node_key,
                    file_path=node.file_node.file_path,
                ),
            )
        elif node.HasField('asset_node'):
            uid = get_uid(dgraph_client, node.asset_node.node_key)

            return NodeView(
                dgraph_client,
                node.asset_node.node_key,
                uid,
                AssetView(
                    dgraph_client=dgraph_client,
                    uid=uid,
                    node_key=node.asset_node.node_key,
                ),
            )
        elif node.HasField("ip_address_node"):
            uid = get_uid(dgraph_client, node.ip_address_node.node_key)

            return NodeView(
                dgraph_client,
                node.ip_address_node.node_key,
                uid,
                IpAddressView(
                    dgraph_client=dgraph_client,
                    uid=uid,
                    node_key=node.ip_address_node.node_key,
                    node_type="IpAddress",
                ),
            )
        elif node.HasField("ip_port_node"):
            uid = get_uid(dgraph_client, node.ip_port_node.node_key)

            return NodeView(
                dgraph_client,
                node.ip_port_node.node_key,
                uid,
                IpPortView(
                    dgraph_client=dgraph_client,
                    uid=uid,
                    node_key=node.ip_port_node.node_key,
                    node_type="IpPort",
                ),
            )
        elif node.HasField("process_outbound_connection_node"):
            uid = get_uid(dgraph_client, node.process_outbound_connection_node.node_key)
            return NodeView(
                dgraph_client,
                node.process_outbound_connection_node.node_key,
                uid,
                ProcessOutboundConnectionView(
                    dgraph_client,
                    node.process_outbound_connection_node.node_key,
                    uid,
                    "ProcessOutboundConnection",
                ),
            )
        elif node.HasField("process_inbound_connection_node"):
            uid = get_uid(dgraph_client, node.process_inbound_connection_node.node_key)
            return NodeView(
                dgraph_client,
                node.process_inbound_connection_node.node_key,
                uid,
                ProcessInboundConnectionView(
                    dgraph_client,
                    node.process_inbound_connection_node.node_key,
                    uid,
                    "ProcessInboundConnection",
                ),
            )
        elif node.HasField("ip_connection_node"):
            uid = get_uid(dgraph_client, node.ip_connection_node.node_key)
            return NodeView(
                dgraph_client,
                node.ip_connection_node.node_key,
                uid,
                IpConnectionView(
                    dgraph_client,
                    node.ip_connection_node.node_key,
                    uid,
                    "IpConnection",
                ),
            )
        elif node.HasField("network_connection_node"):
            uid = get_uid(dgraph_client, node.network_connection_node.node_key)
            return NodeView(
                dgraph_client,
                node.network_connection_node.node_key,
                uid,
                NetworkConnectionView(
                    dgraph_client,
                    node.network_connection_node.node_key,
                    uid,
                    "NetworkConnection",
                ),
            )
        elif node.HasField("dynamic_node"):
            uid = get_uid(dgraph_client, node.dynamic_node.node_key)

            return NodeView(
                dgraph_client,
                node.dynamic_node.node_key,
                uid,
                DynamicNodeView(
                    dgraph_client=dgraph_client,
                    node_key=node.dynamic_node.node_key,
                    uid=uid,
                    node_type=node.dynamic_node.node_type,
                ),
            )
        else:
            raise Exception(f"Invalid Node Type : {node}")

    def as_process(self) -> Optional["ProcessView"]:
        if isinstance(self.node, ProcessView):
            return self.node
        return None

    def as_file(self) -> Optional["FileView"]:
        if isinstance(self.node, FileView):
            return self.node
        return None

    def as_dynamic(self) -> Optional["DynamicNodeView"]:
        if isinstance(self.node, DynamicNodeView):
            return self.node
        return None

    @staticmethod
    def _get_property_types() -> Mapping[str, "PropertyT"]:
        return {}

    @staticmethod
    def _get_forward_edge_types() -> Mapping[str, "EdgeViewT"]:
        return {}

    @staticmethod
    def _get_reverse_edge_types() -> Mapping[str, Tuple["EdgeViewT", str]]:
        pass

    def _get_properties(self) -> Mapping[str, "Property"]:
        return self.node._get_properties()

    def _get_forward_edges(self) -> "Mapping[str, ForwardEdgeView]":
        return self.node._get_forward_edges()

    def _get_reverse_edges(self) -> "Mapping[str,  ReverseEdgeView]":
        return self.node._get_reverse_edges()

    def to_adjacency_list(self) -> Dict[str, Any]:
        all_nodes = flatten_nodes(self.node)
        node_dicts = defaultdict(dict)  # type: Dict[str, Dict[str, Any]]
        edges = defaultdict(list)  # type: Dict[str, List[Dict[str, Any]]]
        for i, node in enumerate(all_nodes):

            node_dict = node.to_dict()
            node_dicts[node_dict["node"]["node_key"]] = node_dict["node"]

            edges[node_dict["node"]["node_key"]].extend(node_dict["edges"])

        return {"nodes": node_dicts, "edges": edges}


from grapl_analyzerlib.nodes.comparators import PropertyFilter
from grapl_analyzerlib.nodes.types import PropertyT, Property

from grapl_analyzerlib.nodes.file_node import FileView
from grapl_analyzerlib.nodes.process_node import ProcessView
from grapl_analyzerlib.nodes.dynamic_node import DynamicNodeView
from grapl_analyzerlib.nodes.viewable import ForwardEdgeView, EdgeViewT, ReverseEdgeView
