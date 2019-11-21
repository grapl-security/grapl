import json
from collections import defaultdict
from typing import Optional, TypeVar, Tuple, Mapping, Any, Union, Type, Dict, List, Set

from pydgraph import DgraphClient

from grapl_analyzerlib.graph_description_pb2 import NodeDescription
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
        res = txn.query(
            query, variables={'$a': node_key}
        )
        res = json.loads(res.json)

        if isinstance(res['res'], list):
            return res['res'][0]['uid']
        else:
            return res['res']['uid']

    finally:
        txn.discard()

def raw_node_from_uid(dgraph_client: DgraphClient, uid: str) -> Optional[Dict[str, Any]]:
    query = f"""
        {{
            res(func: uid("{uid}"), first: 1) {{
                uid,
                expand(_forward_),
                node_type: dgraph.type
            }}
        }}
        """

    txn = dgraph_client.txn(read_only=True, best_effort=False)
    try:
        res = json.loads(txn.query(query).json)['res']
    finally:
        txn.discard()
    if not res:
        return None
    else:
        if isinstance(res, list):
            node_type = res[0].get('node_type')
            if node_type:
                res[0]['node_type'] = res[0]['node_type'][0]
            else:
                print(f"WARN: node_type missing from {uid}")

            return res[0]
        else:
            node_type = res.get('node_type')
            if node_type:
                res['node_type'] = res['node_type'][0]
            else:
                print(f"WARN: node_type missing from {uid}")

            return res


def raw_node_from_node_key(dgraph_client: DgraphClient, node_key: str) -> Optional[Dict[str, Any]]:
    query = f"""
        {{
            res(func: eq(node_key, "{node_key}"), first: 1) {{
                uid,
                expand(_forward_),
                node_type: dgraph.type
            }}
        }}
        """

    txn = dgraph_client.txn(read_only=True, best_effort=False)
    try:
        res = json.loads(txn.query(query).json)['res']
    finally:
        txn.discard()
    if not res:
        return None

    try:
        if isinstance(res, list):
            node_type = res[0].get('node_type')
            if node_type:
                res[0]['node_type'] = res[0]['node_type'][0]
            else:
                print(f"WARN: node_type missing from {node_key}")

            return res[0]
        else:
            node_type = res.get('node_type')
            if node_type:
                res['node_type'] = res['node_type'][0]
            else:
                print(f"WARN: node_type missing from {node_key}")

            return res
    except Exception as e:
        print(f"WARN: raw_node_from_node_key {node_key} {res} {e}")
        raise e


def flatten_nodes(
        root: Viewable
) -> List[Viewable]:
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


class _NodeQuery(Queryable[T]):
    def __init__(self) -> None:
        super(_NodeQuery, self).__init__(_NodeView)

    def _get_unique_predicate(self) -> 'Optional[Tuple[str, PropertyT]]':
        return None

    def _get_node_type_name(self) -> Optional[str]:
        return None

    def _get_property_filters(self) -> Mapping[str, 'PropertyFilter[Property]']:
        return self.dynamic_property_filters

    def _get_forward_edges(self) -> Mapping[str, "Queryable[T]"]:
        return self.dynamic_forward_edge_filters

    def _get_reverse_edges(self) -> Mapping[str, Tuple["Queryable[T]", str]]:
        return self.dynamic_reverse_edge_filters

    def query(
            self,
            dgraph_client: DgraphClient,
            contains_node_key: Optional[str] = None,
            first: Optional[int] = 1000,
    ) -> List['NodeView']:
        return self._query(
            dgraph_client,
            contains_node_key,
            first
        )

    def query_first(
            self, dgraph_client: DgraphClient, contains_node_key: Optional[str] = None
    ) -> Optional['NodeView']:
        return self._query_first(dgraph_client, contains_node_key)


class _NodeView(Viewable[T]):
    def __init__(
            self,
            dgraph_client: DgraphClient,
            node_key: str,
            uid: str,
            node: Union["ProcessView", "FileView", "ExternalIpView", "DynamicNodeView"]
    ):
        super(_NodeView, self).__init__(dgraph_client=dgraph_client, node_key=node_key, uid=uid)
        self.node = node


    @staticmethod
    def from_view(v: Union["ProcessView", "FileView", "ExternalIpView", "DynamicNodeView", "NodeView"]):
        if isinstance(v, _NodeView):
            return v
        try:
            return NodeView(
                v.dgraph_client,
                v.node_key,
                v.uid,
                v,
            )
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
                print(f'ERROR from_node_key failed: {node_key} {res}')
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
                print(f'ERROR from_node_uid failed: {uid} {res}')
                raise e

    @classmethod
    def from_dict(cls: Type['Viewable[T]'], dgraph_client: DgraphClient, d: Dict[str, Any]) -> 'NodeView':

        node_type = d.get('node_type', d.get('dgraph.type', ''))  # type: Optional[str]
        if isinstance(node_type, list):
            node_type = node_type[0]

        _d = raw_node_from_uid(dgraph_client, d.get('uid'))
        if _d:
            d = {**d, **_d}


        if d.get('process_id', d.get('process_name')) or node_type == 'Process':
            node = ProcessView.from_dict(dgraph_client, d)
        elif d.get('file_path') or node_type == 'File':
            node = FileView.from_dict(dgraph_client, d)
        elif d.get('external_ip') or node_type == 'ExternalIp':
            node = ExternalIpView.from_dict(dgraph_client, d)
        elif node_type:
            node = DynamicNodeView.from_dict(dgraph_client, d)
        else:
            raise Exception(f'Invalid scoped node type: {d}')

        assert (
                isinstance(node, _ProcessView) or
                isinstance(node, _FileView) or
                isinstance(node, _ExternalIpView) or
                isinstance(node, _DynamicNodeView)
        )

        return NodeView(
            dgraph_client=dgraph_client,
            node_key=node.node_key,
            uid=node.uid,
            node=node
        )

    @staticmethod
    def from_proto(dgraph_client: DgraphClient, node: NodeDescription) -> 'NodeView':

        if node.HasField("process_node"):
            uid = get_uid(dgraph_client, node.process_node.node_key)
            assert uid

            return NodeView(
                dgraph_client, node.process_node.node_key, uid,
                ProcessView(
                    dgraph_client=dgraph_client,
                    uid=uid,
                    node_key=node.process_node.node_key,
                    process_id=node.process_node.process_id,
                )
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
                )
            )
        elif node.HasField("ip_address_node"):
            uid = get_uid(dgraph_client, node.ip_address_node.node_key)

            return NodeView(
                dgraph_client,
                node.ip_address_node.node_key,
                uid,
                ExternalIpView(
                    dgraph_client=dgraph_client,
                    uid=uid,
                    node_key=node.ip_address_node.node_key,
                )
            )
        elif node.HasField("outbound_connection_node"):
            # uid = get_uid(dgraph_client, node.outbound_connection_node.node_key)
            raise NotImplementedError
            # return NodeView(
            #     dgraph_client, node.outbound_connection_node.node_key, uid,
            #     OutboundConnectionView(
            #         dgraph_client, node.outbound_connection_node.node_key, uid
            #     )
            # )
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
                )
            )
        else:
            raise Exception(f"Invalid Node Type : {node}")

    def as_process(self) -> Optional['ProcessView']:
        if isinstance(self.node, _ProcessView):
            return self.node
        return None

    def as_file(self) -> Optional['FileView']:
        if isinstance(self.node, _FileView):
            return self.node
        return None

    def as_dynamic(self) -> Optional['DynamicNodeView']:
        if isinstance(self.node, _DynamicNodeView):
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

    def _get_properties(self) -> Mapping[str, 'Property']:
        return self._get_properties()

    def _get_forward_edges(self) -> 'Mapping[str, _ForwardEdgeView[T]]':
        return self._get_forward_edges()

    def _get_reverse_edges(self) -> 'Mapping[str,  _ReverseEdgeView[T]]':
        return self._get_reverse_edges()

    def to_adjacency_list(self) -> Dict[str, Any]:
        all_nodes = flatten_nodes(self.node)
        node_dicts = defaultdict(dict)  # type: Dict[str, Dict[str, Any]]
        edges = defaultdict(list)  # type: Dict[str, List[Dict[str, Any]]]
        for i, node in enumerate(all_nodes):

            node_dict = node.to_dict()
            node_dicts[node_dict["node"]["node_key"]] = node_dict["node"]

            edges[node_dict["node"]["node_key"]].extend(node_dict["edges"])

        return {"nodes": node_dicts, "edges": edges}

NodeQuery = _NodeQuery[Any]
NodeView = _NodeView[Any]

from grapl_analyzerlib.nodes.comparators import PropertyFilter
from grapl_analyzerlib.nodes.types import PropertyT, Property
from grapl_analyzerlib.prelude import ProcessView, FileView, ExternalIpView, DynamicNodeView
from grapl_analyzerlib.nodes.external_ip_node import _ExternalIpView
from grapl_analyzerlib.nodes.file_node import _FileView
from grapl_analyzerlib.nodes.process_node import _ProcessView
from grapl_analyzerlib.nodes.dynamic_node import _DynamicNodeView
from grapl_analyzerlib.nodes.viewable import _ForwardEdgeView, EdgeViewT, _ReverseEdgeView
