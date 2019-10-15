import json
from collections import defaultdict
from copy import deepcopy
from typing import Iterator, TypeVar, Callable, Type, Iterable
from typing import Optional, List, Dict, Any, Union
from typing import Tuple

from pydgraph import DgraphClient

from grapl_analyzerlib import graph_description_pb2
from grapl_analyzerlib.querying import (
    PropertyFilter,
    StrCmp,
    EdgeFilter,
    Has,
    Cmp,
    _str_cmps,
    _int_cmps,
    Queryable,
    Not,
    Eq,
    V,
)
from grapl_analyzerlib.querying import Viewable
from grapl_analyzerlib.querying import flatten_nodes

PV = TypeVar("PV", bound="ProcessView")
PQ = TypeVar("PQ", bound="ProcessQuery")

FV = TypeVar("FV", bound="FileView")
FQ = TypeVar("FQ", bound="FileQuery")

OCV = TypeVar("OCV", bound="outbound_connection_node.OutboundConnectionView")
OCQ = TypeVar("OCQ", bound="outbound_connection_node.OutboundConnectionQuery")

EIPV = TypeVar("EIPV", bound="external_ip_node.ExternalIpView")
EIPQ = TypeVar("EIPQ", bound="external_ip_node.ExternalIpQuery")

N = TypeVar("N", bound="entities.NodeView")
S = TypeVar("S", bound="entities.SubgraphView")

DNQ = TypeVar("DNQ", bound="dynamic_node.DynamicNodeQuery")
DNV = TypeVar("DNV", bound="dynamic_node.DynamicNodeView")

PluginNodeView = TypeVar("PluginNodeView")

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
        return res['res'][0]['uid']

    finally:
        txn.discard()


class EdgeView(object):
    def __init__(
        self, from_neighbor_key: str, to_neighbor_key: str, edge_name: str
    ) -> None:
        self.from_neighbor_key = from_neighbor_key
        self.to_neighbor_key = to_neighbor_key
        self.edge_name = edge_name


class NodeView(Viewable):
    def get_property_tuples(self) -> List[Tuple[str, Any]]:
        return []

    def get_edge_tuples(self) -> List[Tuple[str, Any]]:
        return []

    def __init__(
            self,
            dgraph_client: DgraphClient,
            node_key: str,
            uid: str,
            node: Union["PV", "FV", "EIPV", "OCV", "DNV"]
    ):
        super().__init__(dgraph_client, node_key, uid)
        self.node = node

    @staticmethod
    def get_property_types() -> List[Tuple[str, Callable[[Any], Union[str, int]]]]:
        return []

    @staticmethod
    def get_edge_types() -> List[Tuple[str, Union[List[Type[V]], Type[V]]]]:
        return []

    @staticmethod
    def from_raw(dgraph_client: DgraphClient, node: Any) -> "N":
        if node.HasField("process_node"):
            uid = get_uid(dgraph_client, node.process_node.node_key)

            return NodeView(
                dgraph_client, node.process_node.node_key, uid,
                ProcessView(dgraph_client, node.process_node.node_key, uid)
            )
        elif node.HasField("file_node"):
            uid = get_uid(dgraph_client, node.file_node.node_key)

            return NodeView(
                dgraph_client, node.file_node.node_key, uid,
                FileView(dgraph_client, node.file_node.node_key, uid)
            )
        elif node.HasField("ip_address_node"):
            uid = get_uid(dgraph_client, node.ip_address_node.node_key)

            return NodeView(
                dgraph_client, node.ip_address_node.node_key, uid,
                ExternalIpView(dgraph_client, node.ip_address_node.node_key, uid)
            )
        elif node.HasField("outbound_connection_node"):
            uid = get_uid(dgraph_client, node.outbound_connection_node.node_key)

            return NodeView(
                dgraph_client, node.outbound_connection_node.node_key, uid,
                OutboundConnectionView(
                    dgraph_client, node.outbound_connection_node.node_key, uid
                )
            )
        elif node.HasField("dynamic_node"):
            uid = get_uid(dgraph_client, node.dynamic_node.node_key)

            return NodeView(
                dgraph_client,
                node.dynamic_node.node_key,
                uid,
                DynamicNodeView(
                    dgraph_client,
                    node.dynamic_node.node_key,
                    node.dynamic_node.node_type,
                    uid,
                )
            )
        else:
            raise Exception("Invalid Node Type")

    @staticmethod
    def from_view(v: 'Viewable'):
        if isinstance(v, NodeView):
            return v
        return NodeView(v.dgraph_client, v.node_key, v.uid, v)

    def get_node_key(self) -> str:
        return self.node.node_key

    def as_process_view(self) -> Optional["PV"]:
        if isinstance(self.node, ProcessView):
            return self.node
        return None

    def as_file_view(self) -> Optional["FV"]:
        if isinstance(self.node, FileView):
            return self.node
        return None

    def as_dynamic_node(self) -> Optional["DNV"]:
        if isinstance(self.node, DynamicNodeView):
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
            node_dicts[node_dict["node"]["node_key"]] = node_dict["node"]

            edges[node_dict["node"]["node_key"]].extend(node_dict["edges"])

        return {"nodes": node_dicts, "edges": edges}


class SubgraphView(object):
    def __init__(
        self, nodes: Dict[str, NodeView], edges: Dict[str, List[EdgeView]]
    ) -> None:
        self.nodes = nodes
        self.edges = edges

    @staticmethod
    def from_proto(dgraph_client: DgraphClient, s: bytes) -> "S":
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

    def process_iter(self) -> Iterator["PV"]:
        for node in self.nodes.values():
            maybe_node = node.as_process_view()
            if maybe_node:
                yield maybe_node

    def file_iter(self) -> Iterator["FV"]:
        for node in self.nodes.values():
            maybe_node = node.as_file_view()
            if maybe_node:
                yield maybe_node


class DynamicNodeQuery(Queryable):
    def __init__(self, node_type: Optional[str], view_type: Type[V]) -> None:
        super(DynamicNodeQuery, self).__init__(view_type)
        self.node_type = node_type
        self.view_type = view_type

        # Dict of property name to its associated filters
        self.property_filters = defaultdict(list)  # type: Dict[str, PropertyFilter]

        # Dict of edge name to associated filters
        self.edge_filters = dict()
        self.reverse_edge_filters = dict()

    def get_unique_predicate(self) -> Optional[str]:
        return None

    def with_property_str_filter(
        self, prop_name: str, eq=StrCmp, contains=StrCmp, ends_with=StrCmp
    ) -> "DNQ":
        self.property_filters[prop_name].extend(
            _str_cmps(prop_name, eq, contains, ends_with)
        )
        return self

    def with_property_int_filter(
        self,
        prop_name: str,
        eq: Optional[Union[int, List, Not, List[Not]]] = None,
        gt: Optional[Union[int, List, Not, List[Not]]] = None,
        lt: Optional[Union[int, List, Not, List[Not]]] = None,
    ) -> "DNQ":
        self.property_filters[prop_name].extend(_int_cmps(prop_name, eq, gt, lt))
        return self

    def with_edge_filter(self, edge: str, edge_filter: EdgeFilter) -> "DNQ":
        self.edge_filters[edge] = edge_filter
        return self

    def with_reverse_edge_filter(self, edge: str, edge_filter: EdgeFilter) -> "DNQ":
        self.reverse_edge_filters[edge] = edge_filter
        return self

    # Querable Interface Implementation
    def get_node_type_name(self) -> Optional[str]:
        return self.node_type

    def get_node_key_filter(self) -> PropertyFilter:
        return [[self._node_key]]

    def get_uid_filter(self) -> PropertyFilter:
        if isinstance(self._uid, Eq):
            return [[self._uid]]
        return []

    def get_properties(self) -> List[Tuple[str, PropertyFilter]]:
        properties = [(p, f) for (p, f) in self.property_filters.items()]
        properties.append(("node_key", self.get_node_key_filter()))
        properties.append(("uid", self.get_uid_filter()))
        return properties

    def get_forward_edges(self) -> List[Tuple[str, Any]]:
        return [t for t in self.edge_filters.items() if t[1]]

    def get_reverse_edges(self) -> List[Tuple[str, Any]]:
        return [t for t in self.reverse_edge_filters.items() if t[1]]


class DynamicNodeView(Viewable):
    def __init__(
        self,
        dgraph_client: DgraphClient,
        node_key: str,
        node_type: str,
        uid: str,
        asset_id: Optional[str] = None,
    ):
        super(DynamicNodeView, self).__init__(dgraph_client, node_key, uid)
        self.dgraph_client = dgraph_client
        self.node_key = node_key
        self.node_type = node_type
        self.asset_id = asset_id
        self.uid = uid


class FileQuery(Queryable):
    def __init__(self) -> None:
        super(FileQuery, self).__init__(FileView)
        # Attributes
        self._file_name = []  # type: List[List[Cmp]]
        self._asset_id = []  # type: List[List[Cmp]]
        self._file_path = []  # type: List[List[Cmp]]
        self._file_extension = []  # type: List[List[Cmp]]
        self._file_mime_type = []  # type: List[List[Cmp]]
        self._file_size = []  # type: List[List[Cmp]]
        self._file_version = []  # type: List[List[Cmp]]
        self._file_description = []  # type: List[List[Cmp]]
        self._file_product = []  # type: List[List[Cmp]]
        self._file_company = []  # type: List[List[Cmp]]
        self._file_directory = []  # type: List[List[Cmp]]
        self._file_inode = []  # type: List[List[Cmp]]
        self._file_hard_links = []  # type: List[List[Cmp]]
        self._md5_hash = []  # type: List[List[Cmp]]
        self._sha1_hash = []  # type: List[List[Cmp]]
        self._sha256_hash = []  # type: List[List[Cmp]]

        # Edges
        self._creator = None  # type: Optional['PQ']
        self._deleter = None  # type: Optional['PQ']
        self._writers = None  # type: Optional['Q']
        self._readers = None  # type: Optional['PQ']
        self._spawned_from = None  # type: Optional['PQ']

    def with_node_key(self, node_key: Optional[str] = None):
        if node_key:
            self._node_key = Eq("node_key", node_key)
        else:
            self._node_key = Has("node_key")
        return self

    def with_file_name(self, eq=None, contains=None, ends_with=None) -> "FQ":
        self._file_name.extend(_str_cmps("file_name", eq, contains, ends_with))
        return self

    def with_asset_id(self, eq=None, contains=None, ends_with=None) -> "FQ":
        self._asset_id.extend(_str_cmps("asset_id", eq, contains, ends_with))
        return self

    def with_file_path(self, eq=None, contains=None, ends_with=None) -> "FQ":
        self._file_path.extend(_str_cmps("file_path", eq, contains, ends_with))
        return self

    def with_file_extension(self, eq=None, contains=None, ends_with=None) -> "FQ":
        self._file_extension.extend(
            _str_cmps("file_extension", eq, contains, ends_with)
        )
        return self

    def with_file_mime_type(self, eq=None, contains=None, ends_with=None) -> "FQ":
        self._file_mime_type.extend(
            _str_cmps("file_mime_type", eq, contains, ends_with)
        )
        return self

    def with_file_size(self, eq=None, contains=None, ends_with=None) -> "FQ":
        self._file_size.extend(_str_cmps("file_size", eq, contains, ends_with))
        return self

    def with_file_version(self, eq=None, contains=None, ends_with=None) -> "FQ":
        self._file_version.extend(_str_cmps("file_version", eq, contains, ends_with))
        return self

    def with_file_description(self, eq=None, contains=None, ends_with=None) -> "FQ":
        self._file_description.extend(
            _str_cmps("file_description", eq, contains, ends_with)
        )
        return self

    def with_file_product(self, eq=None, contains=None, ends_with=None) -> "FQ":
        self._file_product.extend(_str_cmps("file_product", eq, contains, ends_with))
        return self

    def with_file_company(self, eq=None, contains=None, ends_with=None) -> "FQ":
        self._file_company.extend(_str_cmps("file_company", eq, contains, ends_with))
        return self

    def with_file_directory(self, eq=None, contains=None, ends_with=None) -> "FQ":
        self._file_directory.extend(
            _str_cmps("file_directory", eq, contains, ends_with)
        )
        return self

    def with_file_inode(self, eq=None, contains=None, ends_with=None) -> "FQ":
        self._file_inode.extend(_str_cmps("file_inode", eq, contains, ends_with))
        return self

    def with_file_hard_links(self, eq=None, contains=None, ends_with=None) -> "FQ":
        self._file_hard_links.extend(
            _str_cmps("file_hard_links", eq, contains, ends_with)
        )
        return self

    def with_md5_hash(self, eq=None, contains=None, ends_with=None) -> "FQ":
        self._md5_hash.extend(_str_cmps("md5_hash", eq, contains, ends_with))
        return self

    def with_sha1_hash(self, eq=None, contains=None, ends_with=None) -> "FQ":
        self._sha1_hash.extend(_str_cmps("sha1_hash", eq, contains, ends_with))
        return self

    def with_sha256_hash(self, eq=None, contains=None, ends_with=None) -> "FQ":
        self._sha256_hash.extend(_str_cmps("sha256_hash", eq, contains, ends_with))
        return self

    def with_creator(self, creator: "PQ") -> "FQ":
        creator = deepcopy(creator)
        creator._created_files = self
        self._creator = creator
        return self

    def with_deleter(self, deleter: "PQ") -> "FQ":
        deleter = deepcopy(deleter)
        self._deleter = deleter
        deleter._deleted_files = self
        return self

    def with_writers(self, writers: "PQ") -> "FQ":
        writers = deepcopy(writers)
        self._writers = writers
        return self

    def with_readers(self, readers: "PQ") -> "FQ":
        readers = deepcopy(readers)
        self._readers = readers
        readers._read_files = self
        return self

    def with_spawned_from(self, spawned_from: "PQ") -> "FQ":
        spawned_from = deepcopy(spawned_from)
        self._spawned_from = spawned_from
        spawned_from._bin_file = self
        return self

    def get_unique_predicate(self) -> Optional[str]:
        return "file_path"

    def get_node_type_name(self) -> Optional[str]:
        return None

    def get_node_key_filter(self) -> PropertyFilter:
        return [[self._node_key]]

    def get_uid_filter(self) -> PropertyFilter:
        if isinstance(self._uid, Eq):
            return [[self._uid]]
        return []

    def get_properties(self) -> List[Tuple[str, PropertyFilter]]:
        properties = (
            ("node_key", self.get_node_key_filter()),
            ("uid", self.get_uid_filter()),
            ("file_name", self._file_name),
            ("asset_id", self._asset_id),
            ("file_path", self._file_path),
            ("file_extension", self._file_extension),
            ("file_mime_type", self._file_mime_type),
            ("file_size", self._file_size),
            ("file_version", self._file_version),
            ("file_description", self._file_description),
            ("file_product", self._file_product),
            ("file_company", self._file_company),
            ("file_directory", self._file_directory),
            ("file_inode", self._file_inode),
            ("file_hard_links", self._file_hard_links),
            ("md5_hash", self._md5_hash),
            ("sha1_hash", self._sha1_hash),
            ("sha256_hash", self._sha256_hash),
        )

        return [p for p in properties if p[1]]

    def get_neighbors(self) -> List[Any]:
        neighbors = (
            self._creator,
            self._deleter,
            self._writers,
            self._readers,
            self._spawned_from,
        )

        return [n for n in neighbors if n]

    def get_forward_edges(self) -> List[Tuple[str, Any]]:
        return []

    def get_reverse_edges(self) -> List[Tuple[str, Any, str]]:
        neighbors = (
            ("~created_files", self._creator, "creator"),
            ("~deleted_files", self._deleter, "deleter"),
            ("~wrote_to_files", self._writers, "writers"),
            ("~read_files", self._readers, "readers"),
            ("~bin_file", self._spawned_from, "spawned_from"),
        )

        return [n for n in neighbors if n[1]]


class FileView(Viewable):
    def __init__(
        self,
        dgraph_client: DgraphClient,
        node_key: str,
        uid: Optional[str] = None,
        asset_id: Optional[str] = None,
        file_name: Optional[str] = None,
        file_path: Optional[str] = None,
        file_extension: Optional[str] = None,
        file_mime_type: Optional[str] = None,
        file_size: Optional[int] = None,
        file_version: Optional[str] = None,
        file_description: Optional[str] = None,
        file_product: Optional[str] = None,
        file_company: Optional[str] = None,
        file_directory: Optional[str] = None,
        file_inode: Optional[str] = None,
        file_hard_links: Optional[int] = None,
        md5_hash: Optional[str] = None,
        sha1_hash: Optional[str] = None,
        sha256_hash: Optional[str] = None,
        creator: Optional["PV"] = None,
        deleter: Optional["PV"] = None,
        writers: Optional[List["PV"]] = None,
        readers: Optional[List["PV"]] = None,
        spawned_from: Optional[List["PV"]] = None,
    ) -> None:
        super(FileView, self).__init__(dgraph_client, node_key, uid)
        self.dgraph_client = dgraph_client  # type: DgraphClient
        self.node_key = node_key  # type: Optional[str]
        self.uid = uid  # type: Optional[str]
        self.asset_id = asset_id
        self.file_name = file_name
        self.file_path = file_path
        self.file_extension = file_extension
        self.file_mime_type = file_mime_type
        self.file_size = int(file_size) if file_size else None
        self.file_version = file_version
        self.file_description = file_description
        self.file_product = file_product
        self.file_company = file_company
        self.file_directory = file_directory
        self.file_inode = int(file_inode) if file_inode else None
        self.file_hard_links = file_hard_links
        self.md5_hash = md5_hash
        self.sha1_hash = sha1_hash
        self.sha256_hash = sha256_hash
        self.creator = creator
        self.deleter = deleter
        self.writers = writers
        self.readers = readers
        self.spawned_from = spawned_from


    def get_property_tuples(self) -> List[Tuple[str, Any]]:
        props = (
            ("node_key", self.node_key),
            ("uid", self.uid),
            ("asset_id", self.asset_id),
            ("file_name", self.file_name),
            ("file_path", self.file_path),
            ("file_extension", self.file_extension),
            ("file_mime_type", self.file_mime_type),
            ("file_size", self.file_size),
            ("file_version", self.file_version),
            ("file_description", self.file_description),
            ("file_product", self.file_product),
            ("file_company", self.file_company),
            ("file_directory", self.file_directory),
            ("file_inode", self.file_inode),
            ("file_hard_links", self.file_hard_links),
            ("md5_hash", self.md5_hash),
            ("sha1_hash", self.sha1_hash),
            ("sha256_hash", self.sha256_hash),
        )

        return [p for p in props if p[1]]

    def get_edge_tuples(self) -> List[Tuple[str, Any]]:
        edges = (
            ("creator", self.creator),
            ("deleter", self.deleter),
            ("writers", self.writers),
            ("readers", self.readers),
            ("spawned_from", self.spawned_from),
        )
        return [e for e in edges if e[1]]

    @staticmethod
    def get_property_types() -> List[Tuple[str, Callable[[Any], Union[str, int]]]]:
        return [
            ("asset_id", str),
            ("file_path", str),
            ("file_name", str),
            ("file_extension", str),
            ("file_mime_type", str),
            ("file_size", int),
            ("file_version", str),
            ("file_description", str),
            ("file_product", str),
            ("file_company", str),
            ("file_directory", str),
            ("file_inode", int),
            ("file_hard_links", str),
            ("md5_hash", str),
            ("sha1_hash", str),
            ("sha256_hash", str),
        ]

    @staticmethod
    def get_edge_types() -> List[Tuple[str, Union[List[Type[V]], Type[V]]]]:
        return [
            ("~created_files", ProcessView, "creator"),
            ("~deleted_files", ProcessView, "deleter"),
            ("~wrote_to_files", ProcessView, "writers"),
            ("~read_files", ProcessView, "readers"),
            ("~bin_file", ProcessView, "spawned_from"),
        ]

    def get_file_path(self) -> Optional[str]:
        self_file = (
            FileQuery()
            .with_node_key(self.node_key)
            .with_file_path()
            .query_first(dgraph_client=self.dgraph_client)
        )

        if not self_file:
            return None

        self.file_path = self_file.file_path
        return self.file_path

    def get_file_name(self) -> Optional[str]:
        self_file = (
            FileQuery()
            .with_node_key(self.node_key)
            .with_file_name()
            .query_first(dgraph_client=self.dgraph_client)
        )

        if not self_file:
            return None

        self.file_name = self_file.file_name
        return self.file_name

    def get_file_extension(self) -> Optional[str]:
        self_file = (
            FileQuery()
            .with_node_key(self.node_key)
            .with_file_extension()
            .query_first(dgraph_client=self.dgraph_client)
        )

        if not self_file:
            return None

        self.file_extension = self_file.file_extension
        return self.file_extension

    def get_file_mime_type(self) -> Optional[str]:
        self_file = (
            FileQuery()
            .with_node_key(self.node_key)
            .with_file_mime_type()
            .query_first(dgraph_client=self.dgraph_client)
        )

        if not self_file:
            return None

        self.file_mime_type = self_file.file_mime_type
        return self.file_mime_type

    def get_file_size(self) -> Optional[int]:
        self_file = (
            FileQuery()
            .with_node_key(self.node_key)
            .with_file_size()
            .query_first(dgraph_client=self.dgraph_client)
        )

        if not self_file:
            return None

        if not self_file.file_size:
            return None

        self.file_size = int(self_file.file_size)
        return self.file_size

    def get_file_version(self) -> Optional[str]:
        self_file = (
            FileQuery()
            .with_node_key(self.node_key)
            .with_file_version()
            .query_first(dgraph_client=self.dgraph_client)
        )

        if not self_file:
            return None

        self.file_version = self_file.file_version
        return self.file_version

    def get_file_description(self) -> Optional[str]:
        self_file = (
            FileQuery()
            .with_node_key(self.node_key)
            .with_file_description()
            .query_first(dgraph_client=self.dgraph_client)
        )

        if not self_file:
            return None

        self.file_description = self_file.file_description
        return self.file_description

    def get_file_product(self) -> Optional[str]:
        self_file = (
            FileQuery()
            .with_node_key(self.node_key)
            .with_file_product()
            .query_first(dgraph_client=self.dgraph_client)
        )

        if not self_file:
            return None

        self.file_product = self_file.file_product
        return self.file_product

    def get_file_company(self) -> Optional[str]:
        self_file = (
            FileQuery()
            .with_node_key(self.node_key)
            .with_file_company()
            .query_first(dgraph_client=self.dgraph_client)
        )

        if not self_file:
            return None

        self.file_company = self_file.file_company
        return self.file_company

    def get_file_directory(self) -> Optional[str]:
        self_file = (
            FileQuery()
            .with_node_key(self.node_key)
            .with_file_directory()
            .query_first(dgraph_client=self.dgraph_client)
        )

        if not self_file:
            return None

        self.file_directory = self_file.file_directory
        return self.file_directory

    def get_file_inode(self) -> Optional[str]:
        self_file = (
            FileQuery()
            .with_node_key(self.node_key)
            .with_file_inode()
            .query_first(dgraph_client=self.dgraph_client)
        )

        if not self_file:
            return None

        self.file_inode = self_file.file_inode
        return self.file_inode

    def get_file_hard_links(self) -> Optional[str]:
        self_file = (
            FileQuery()
            .with_node_key(self.node_key)
            .with_file_hard_links()
            .query_first(dgraph_client=self.dgraph_client)
        )

        if not self_file:
            return None

        self.file_hard_links = self_file.file_hard_links
        return self.file_hard_links

    def get_md5_hash(self) -> Optional[str]:
        self_file = (
            FileQuery()
            .with_node_key(self.node_key)
            .with_md5_hash()
            .query_first(dgraph_client=self.dgraph_client)
        )

        if not self_file:
            return None

        self.md5_hash = self_file.md5_hash
        return self.md5_hash

    def get_sha1_hash(self) -> Optional[str]:
        self_file = (
            FileQuery()
            .with_node_key(self.node_key)
            .with_sha1_hash()
            .query_first(dgraph_client=self.dgraph_client)
        )

        if not self_file:
            return None

        self.sha1_hash = self_file.sha1_hash
        return self.sha1_hash

    def get_sha256_hash(self) -> Optional[str]:
        self_file = (
            FileQuery()
            .with_node_key(self.node_key)
            .with_sha256_hash()
            .query_first(dgraph_client=self.dgraph_client)
        )

        if not self_file:
            return None

        self.sha256_hash = self_file.sha256_hash
        return self.sha256_hash

    def get_file_path(self) -> Optional[str]:
        self_file = (
            FileQuery()
            .with_node_key(self.node_key)
            .with_file_path()
            .query_first(dgraph_client=self.dgraph_client)
        )

        if not self_file:
            return None

        self.file_path = self_file.file_path
        return self.file_path

    def get_asset_id(self) -> Optional[str]:
        self_file = (
            FileQuery()
            .with_node_key(self.node_key)
            .with_asset_id()
            .query_first(dgraph_client=self.dgraph_client)
        )

        if not self_file:
            return None

        self.asset_id = self_file.asset_id
        return self.asset_id

    def get_creator(self) -> Optional["PV"]:
        creator = (
            ProcessQuery()
            .with_created_files(FileQuery().with_node_key(self.node_key))
            .query_first(self.dgraph_client)
        )  # type: Optional[ProcessView]

        if not creator:
            return None

        if not creator.created_files:
            return None

        self_node = creator.created_files[0]

        if not self_node:
            return None

        self.creator = self_node.creator
        return self.creator

    def get_spawned_from(self) -> Optional["PV"]:
        self_node = (
            FileQuery().with_node_key(self.node_key)
            .with_spawned_from(ProcessQuery())
            .query_first(self.dgraph_client)
        )  # type: Optional['FileView']

        if not self_node:
            return None

        self.spawned_from = self_node.spawned_from
        return self.spawned_from

    def to_dict(self, root=False) -> Dict[str, Any]:
        node_dict = dict()

        for prop_name, prop in self.get_property_tuples():
            node_dict[prop_name] = prop

        if root:
            node_dict["root"] = True

        return {"node": node_dict, "edges": []}  # TODO: Generate edges


class ProcessQuery(Queryable):
    def __init__(self) -> None:
        super(ProcessQuery, self).__init__(ProcessView)
        # Properties

        self._asset_id = []  # type: List[List[Cmp]]
        self._process_name = []  # type: List[List[Cmp]]
        self._process_command_line = []  # type: List[List[Cmp]]
        self._process_guid = []  # type: List[List[Cmp]]
        self._process_id = []  # type: List[List[Cmp]]
        self._created_timestamp = []  # type: List[List[Cmp]]
        self._terminated_timestamp = []  # type: List[List[Cmp]]
        self._last_seen_timestamp = []  # type: List[List[Cmp]]

        # Edges
        self._bin_file = None  # type: Optional[FQ]
        self._children = None  # type: Optional[PQ]
        self._deleted_files = None  # type: Optional[FQ]
        self._created_files = None  # type: Optional[FQ]
        self._wrote_to_files = None  # type: Optional[FQ]
        self._read_files = None  # type: Optional[FQ]
        self._created_connection = None  # type: Optional['OCQ']

        self._parent = None  # type: Optional[PQ]

        # Meta
        self._first = None  # type: Optional[int]

    def with_node_key(self, eq: Optional[Union[Not, str]] = None):
        if eq:
            self._node_key = Eq("node_key", eq)
        else:
            self._node_key = Has("node_key")
        return self

    def with_uid(self, eq: Union[Not, str]):
        if eq:
            self._uid = Eq("uid", eq)
        return self

    def only_first(self, first: int) -> "PQ":
        self._first = first
        return self

    def with_asset_id(
        self,
        eq: Optional[Union[str, List[str], Not, List[Not]]] = None,
        contains: Optional[Union[str, List[str], Not, List[Not]]] = None,
        ends_with: Optional[Union[str, List[str], Not, List[Not]]] = None,
    ) -> "PQ":
        self._asset_id.extend(_str_cmps("asset_id", eq, contains, ends_with))
        return self

    def with_process_name(
        self,
        eq: Optional[Union[str, List[str], Not, List[Not]]] = None,
        contains: Optional[Union[str, List[str], Not, List[Not]]] = None,
        ends_with: Optional[Union[str, List[str], Not, List[Not]]] = None,
    ) -> "PQ":
        self._process_name.extend(_str_cmps("process_name", eq, contains, ends_with))
        return self

    def with_process_command_line(
        self,
        eq: Optional[Union[str, List[str], Not, List[Not]]] = None,
        contains: Optional[Union[str, List[str], Not, List[Not]]] = None,
        ends_with: Optional[Union[str, List[str], Not, List[Not]]] = None,
    ) -> "PQ":
        self._process_command_line.extend(
            _str_cmps("process_command_line", eq, contains, ends_with)
        )
        return self

    def with_process_guid(
        self,
        eq: Optional[Union[str, List[str], Not, List[Not]]] = None,
        contains: Optional[Union[str, List[str], Not, List[Not]]] = None,
        ends_with: Optional[Union[str, List[str], Not, List[Not]]] = None,
    ) -> "PQ":
        self._process_guid.extend(_str_cmps("process_guid", eq, contains, ends_with))
        return self

    def with_process_id(
        self,
        eq: Optional[Union[str, List[int], Not, List[Not]]] = None,
        gt: Optional[Union[str, List[int], Not, List[Not]]] = None,
        lt: Optional[Union[str, List[int], Not, List[Not]]] = None,
    ) -> "PQ":
        self._process_id.extend(_int_cmps("process_id", eq, gt, lt))
        return self

    def with_created_timestamp(
        self,
        eq: Optional[Union[str, List[int], Not, List[Not]]] = None,
        gt: Optional[Union[str, List[int], Not, List[Not]]] = None,
        lt: Optional[Union[str, List[int], Not, List[Not]]] = None,
    ) -> "PQ":
        self._created_timestamp.extend(_int_cmps("created_timestamp", eq, gt, lt))
        return self

    def with_terminated_timestamp(
        self,
        eq: Optional[Union[str, List[int], Not, List[Not]]] = None,
        gt: Optional[Union[str, List[int], Not, List[Not]]] = None,
        lt: Optional[Union[str, List[int], Not, List[Not]]] = None,
    ) -> "PQ":
        self._terminated_timestamp.extend(_int_cmps("terminated_timestamp", eq, gt, lt))
        return self

    def with_last_seen_timestamp(
        self,
        eq: Optional[Union[str, List[int], Not, List[Not]]] = None,
        gt: Optional[Union[str, List[int], Not, List[Not]]] = None,
        lt: Optional[Union[str, List[int], Not, List[Not]]] = None,
    ) -> "PQ":
        self._last_seen_timestamp.extend(_int_cmps("last_seen_timestamp", eq, gt, lt))
        return self

    def with_parent(self, process: "PQ") -> "PQ":
        process: "PQ" = deepcopy(process)
        process._children = self
        self._parent = process
        return self

    def with_bin_file(self, file: "FQ") -> "PQ":
        file = deepcopy(file)
        file._spawned_from = self
        self._bin_file = file
        return self

    def with_deleted_files(self, file: "FQ") -> "PQ":
        file = deepcopy(file)
        file._deleter = self
        self._deleted_files = file
        return self

    def with_created_files(self, file: "FQ") -> "PQ":
        file = deepcopy(file)
        file._creator = self
        self._created_files = file
        return self

    def with_written_files(self, file: "FQ") -> "PQ":
        file = deepcopy(file)
        file._writers = self
        self._wrote_to_files = file
        return self

    def with_read_files(self, file: "FQ") -> "PQ":
        file = deepcopy(file)
        file._readers = self
        self._read_files = file
        return self

    def with_children(self, children: "PQ") -> "PQ":
        children = deepcopy(children)
        children._parent = self
        self._children = children
        return self

    def with_created_connection(self, outbound_conn: Union["OCQ", "EIPQ"]) -> "PQ":
        outbound_conn = deepcopy(outbound_conn)

        if isinstance(outbound_conn, ExternalIpQuery):
            outbound_conn = OutboundConnectionQuery().with_external_connection(
                outbound_conn
            )
        outbound_conn._connecting_process = self

        self._created_connection = outbound_conn
        return self

    def get_properties(self) -> List[Tuple[str, PropertyFilter]]:
        properties = [
            ("node_key", self.get_node_key_filter()),
            ("uid", self.get_uid_filter()),
            ("asset_id", self._asset_id),
            ("process_name", self._process_name),
            ("process_command_line", self._process_command_line),
            ("process_guid", self._process_guid),
            ("process_id", self._process_id),
            ("created_timestamp", self._created_timestamp),
            ("terminated_timestamp", self._terminated_timestamp),
            ("last_seen_timestamp", self._last_seen_timestamp),
        ]

        return [p for p in properties if p[1]]

    def get_node_type_name(self) -> Optional[str]:
        return None

    def get_unique_predicate(self) -> Optional[str]:
        return "process_id"

    def get_node_key_filter(self) -> PropertyFilter:
        return [[self._node_key]]

    def get_uid_filter(self) -> PropertyFilter:
        if isinstance(self._uid, Eq):
            return [[self._uid]]
        return []

    def get_forward_edges(self) -> List[Tuple[str, Any]]:
        edges = [
            ("bin_file", self._bin_file),
            ("children", self._children),
            ("deleted_files", self._deleted_files),
            ("created_files", self._created_files),
            ("wrote_to_files", self._wrote_to_files),
            ("read_files", self._read_files),
            ("created_connection", self._created_connection),
        ]

        return [e for e in edges if e[1]]

    def get_reverse_edges(self) -> List[Tuple[str, Any, str]]:
        edges = [("~children", self._parent, 'parent')]

        return [e for e in edges if e[1]]


class ProcessView(Viewable):
    def __init__(
        self,
        dgraph_client: DgraphClient,
        node_key: str,
        uid: str,
        asset_id: Optional[str] = None,
        process_name: Optional[str] = None,
        process_command_line: Optional[str] = None,
        process_guid: Optional[str] = None,
        process_id: Optional[str] = None,
        created_timestamp: Optional[str] = None,
        terminated_timestamp: Optional[str] = None,
        last_seen_timestamp: Optional[str] = None,
        bin_file: Optional["FV"] = None,
        parent: Optional["PV"] = None,
        children: Optional[List["PV"]] = None,
        deleted_files: Optional[List["FV"]] = None,
        created_files: Optional[List["FV"]] = None,
        read_files: Optional[List["FV"]] = None,
        created_connections: Optional[List["EIPV"]] = None,
        **kwargs,
    ) -> None:
        super(ProcessView, self).__init__(dgraph_client, node_key, uid)

        self.dgraph_client = dgraph_client  # type: DgraphClient
        self.node_key = node_key  # type: str
        self.uid = uid  # type: Optional[str]
        self.asset_id = asset_id
        self.process_command_line = process_command_line
        self.process_guid = process_guid
        self.process_id = process_id
        self.created_timestamp = created_timestamp
        self.terminated_timestamp = terminated_timestamp
        self.last_seen_timestamp = last_seen_timestamp
        self.process_name = process_name  # type: Optional[str]

        self.bin_file = bin_file  # type: Optional['FV']
        self.children = children or []  # type: List['PV']
        self.parent = parent  # type: Optional['PV']
        self.deleted_files = deleted_files  # type: Optional[List['FV']]
        self.created_files = created_files  # type: Optional[List['FV']]
        self.read_files = read_files  # type: Optional[List['FV']]
        self.created_connections = created_connections  # type: Optional[List['EIPV']]

    @staticmethod
    def get_property_types() -> List[Tuple[str, Callable[[Any], Union[str, int]]]]:
        return [
            ("asset_id", str),
            ("process_name", str),
            ("process_command_line", str),
            ("process_guid", str),
            ("process_id", str),
            ("created_timestamp", int),
            ("terminated_timestamp", int),
            ("last_seen_timestamp", int),
        ]

    @staticmethod
    def get_edge_types() -> List[Tuple[str, Union[List[Type[V]], Type[V]]]]:
        return [
            ("bin_file", FileView),
            ("children", [ProcessView]),
            ("deleted_files", [FileView]),
            ("created_files", [FileView]),
            ("read_files", [FileView]),
            ("created_connections", [ExternalIpView]),
            ("~children", ProcessView, "parent"),
        ]

    def get_asset_id(self) -> Optional[str]:
        self_process = (
            ProcessQuery()
            .with_node_key(self.node_key)
            .with_asset_id()
            .query_first(dgraph_client=self.dgraph_client)
        )

        if not self_process:
            return None

        self.asset_id = self_process.asset_id
        return self.asset_id

    def get_created_timestamp(self) -> Optional[int]:
        self_process = (
            ProcessQuery()
                .with_node_key(self.node_key)
                .with_created_timestamp()
                .query_first(dgraph_client=self.dgraph_client)
        )

        if not self_process:
            return None

        self.created_timestamp = self_process.created_timestamp
        return self.created_timestamp

    def get_terminated_timestamp(self) -> Optional[int]:
        self_process = (
                ProcessQuery()
                    .with_node_key(self.node_key)
                    .with_terminated_timestamp()
                    .query_first(dgraph_client=self.dgraph_client)
            )

        if not self_process:
            return None

        self.terminated_timestamp = self_process.terminated_timestamp
        return self.terminated_timestamp

    def get_last_seen_timestamp(self) -> Optional[int]:
        self_process = (
            ProcessQuery()
                .with_node_key(self.node_key)
                .with_last_seen_timestamp()
                .query_first(dgraph_client=self.dgraph_client)
        )

        if not self_process:
            return None

        self.last_seen_timestamp = self_process.last_seen_timestamp
        return self.last_seen_timestamp

    def get_descendents(self, max=5, limit=50) -> List["ProcessView"]:
        descendents = []
        self.children = self.get_children()
        for child in self.children:
            descendents.append(child)
            # noinspection PyProtectedMember
            child._get_descendents(descendents, max, limit)
        return descendents

    def _get_descendents(self, descendents, max_depth, limit, depth=0):
        if len(descendents) >= limit:
            return
        if depth >= max_depth:
            return
        self.children = self.get_children()
        for child in self.children:
            descendents.append(child)
            # noinspection PyProtectedMember
            child._get_descendents(descendents, max_depth, limit, depth + 1)

    def traverse_descendents(self) -> Iterable["ProcessView"]:
        if not self.children:
            self.get_descendents()
        for child in self.children:
            yield child
            for c in child.traverse_descendents():
                yield c

    def get_process_name(self) -> Optional[str]:
        self_process = (
            ProcessQuery()
            .with_node_key(self.node_key)
            .with_process_name()
            .query_first(dgraph_client=self.dgraph_client)
        )

        if not self_process:
            return None

        self.process_name = self_process.process_name
        return self.process_name

    def get_process_id(self) -> Optional[str]:
        self_process = (
            ProcessQuery()
                .with_node_key(self.node_key)
                .with_process_id()
                .query_first(dgraph_client=self.dgraph_client)
        )

        if not self_process:
            return None

        self.process_id = self_process.process_id
        return self.process_id

    def get_process_command_line(self) -> Optional[str]:
        self_process = (
            ProcessQuery()
            .with_node_key(self.node_key)
            .with_process_command_line()
            .query_first(dgraph_client=self.dgraph_client)
        )

        if not self_process:
            return None

        self.process_command_line = self_process.process_command_line
        return self.process_command_line

    def get_parent(self) -> Optional["PV"]:
        self_node = (
            ProcessQuery()
            .with_node_key(self.node_key)
            .with_parent(ProcessQuery())
            .query_first(self.dgraph_client)
        )

        if not self_node:
            return None

        if not self_node.parent:
            return None

        self.parent = self_node.parent
        self.parent.get_process_name()
        self.parent.get_created_timestamp()
        self.parent.get_last_seen_timestamp()

        return self.parent

    def get_created_files(self) -> Optional[List["FV"]]:
        self_node = (
            ProcessQuery()
                .with_node_key(self.node_key)
                .with_created_files(FileQuery().with_node_key())
                .query_first(self.dgraph_client)
        )

        if not self_node:
            return None

        self.created_files = self_node.created_files
        return self.created_files

    def get_children(self) -> List["PV"]:
        self_node = (
            ProcessQuery()
            .with_node_key(self.node_key)
            .with_children(ProcessQuery().with_node_key())
            .query_first(self.dgraph_client)
        )  # type: Optional['PV']

        if not self_node:
            return []

        self.children = self_node.children or []

        # Get the process name by default
        for child in self.children:
            child.get_process_name()
            child.get_created_timestamp()
            child.get_last_seen_timestamp()

        return self.children

    def get_bin_file(self) -> Optional["FV"]:
        self_node = (
            ProcessQuery()
            .with_node_key(self.node_key)
            .with_bin_file(FileQuery())
            .query_first(self.dgraph_client)
        )
        if not self_node:
            return None

        self.bin_file = self_node.bin_file
        return self.bin_file

    def get_created_connections(self) -> List["OCV"]:
        self_node = (
            ProcessQuery()
            .with_node_key(self.node_key)
            .with_created_connection(OutboundConnectionQuery())
            .query_first(self.dgraph_client)
        )
        if not self_node:
            return []

        self.created_connections = self_node.created_connections
        return self.created_connections

    def get_external_connections(self) -> List["OCV"]:
        self_node = (
            ProcessQuery()
            .with_node_key(self.node_key)
            .with_created_connection(ExternalIpQuery())
            .query_first(self.dgraph_client)
        )
        if not self_node:
            return []

        self.created_connections = self_node.created_connections
        return self.created_connections

    def get_deleted_files(self) -> Optional[List["FV"]]:
        deleted_files = (
            ProcessQuery()
            .with_node_key(self.node_key)
            .with_deleted_files(FileQuery().with_node_key())
            .query_first(self.dgraph_client)
        )

        if not deleted_files:
            return None

        self.deleted_files = deleted_files[0].deleted_files
        return self.deleted_files

    def get_read_files(self) -> Optional[List["FV"]]:
        self_node = (
            ProcessQuery()
            .with_node_key(self.node_key)
            .with_read_files(FileQuery().with_node_key())
            .query_first(self.dgraph_client)
        )

        if not self_node:
            return None

        if not self_node.read_files:
            return None

        self.read_files = self_node.read_files
        return self.read_files

    def get_neighbors(self) -> List[Any]:
        neighbors = (self.parent, self.bin_file, self.children, self.deleted_files)

        return [n for n in neighbors if n]

    def get_property_tuples(self) -> List[Tuple[str, Any]]:
        props = (
            ("node_key", self.node_key),
            ("uid", self.uid),
            ("asset_id", self.asset_id),
            ("process_command_line", self.process_command_line),
            ("process_guid", self.process_guid),
            ("process_id", self.process_id),
            ("created_timestamp", self.created_timestamp),
            ("terminated_timestamp", self.terminated_timestamp),
            ("last_seen_timestamp", self.last_seen_timestamp),
            ("process_name", self.process_name),
        )

        return [p for p in props if p[1]]

    def get_edge_tuples(self):
        edges = (
            ("bin_file", self.bin_file),
            ("children", self.children),
            ("deleted_files", self.deleted_files),
            ("created_files", self.created_files),
            ("read_files", self.read_files),
            ("created_connections", self.created_connections),
            ("children", self.children),
        )
        return [e for e in edges if e[1]]

    def to_dict(self, root=False) -> Dict[str, Any]:
        node_dict = dict()
        edges = []
        node_dict["node_type"] = "Process"

        for prop_name, prop in self.get_property_tuples():
            node_dict[prop_name] = prop

        for edge_name, edge in self.get_edge_tuples():
            if isinstance(edge, list):
                for e in edge:
                    node_dict[edge_name] = e.node_key
                    edges.append(
                        {"from": self.node_key, "edge_name": edge_name, "to": e.node_key}
                    )
            else:
                node_dict[edge_name] = edge.node_key

                edges.append(
                    {"from": self.node_key, "edge_name": edge_name, "to": edge.node_key}
                )

        if root:
            node_dict["root"] = True

        return {"node": node_dict, "edges": edges}


class OutboundConnectionQuery(Queryable):
    def __init__(self) -> None:
        super(OutboundConnectionQuery, self).__init__(OutboundConnectionView)

        self._create_time = []  # type: List[List[Cmp]]
        self._terminate_time = []  # type: List[List[Cmp]]
        self._last_seen_time = []  # type: List[List[Cmp]]
        self._ip = []  # type: List[List[Cmp]]
        self._port = []  # type: List[List[Cmp]]

        # self._internal_connection = None  # type: Optional[Any]
        self._external_connection = None  # type: 'Optional[EIPQ]'
        self._connecting_process = None  # type: 'Optional[PQ]'

    def get_unique_predicate(self) -> Optional[str]:
        return "port"

    def get_node_type_name(self) -> Optional[str]:
        return None

    def get_node_key_filter(self) -> PropertyFilter:
        return [[self._node_key]]

    def get_uid_filter(self) -> PropertyFilter:
        if isinstance(self._uid, Eq):
            return [[self._uid]]
        return []

    def get_properties(self) -> List[Tuple[str, PropertyFilter]]:
        properties = (
            ("node_key", self.get_node_key_filter()),
            ("uid", self.get_uid_filter()),
            ("create_time", self._create_time),
            ("terminate_time", self._terminate_time),
            ("last_seen_time", self._last_seen_time),
            ("ip", self._ip),
            ("port", self._port),
        )

        return [p for p in properties if p[1]]

    def get_forward_edges(self) -> List[Tuple[str, Any]]:
        edges = [("external_connection", self._external_connection)]
        return [e for e in edges if e[1]]

    def get_reverse_edges(self) -> List[Tuple[str, Any]]:
        edges = [("~created_connection", self._connecting_process)]
        return [e for e in edges if e[1]]

    def with_external_connection(self, external_ip: "EIPQ") -> "OCQ":
        external_ip = deepcopy(external_ip)
        external_ip._connections_from = self
        self._external_connection = external_ip
        return self

    def with_connecting_process(self, process: "PQ") -> "OCQ":
        process = deepcopy(process)
        process._created_connection = self
        self._connecting_process = process
        return self


class OutboundConnectionView(Viewable):
    def __init__(
        self,
        dgraph_client: DgraphClient,
        node_key: str,
        uid: Optional[str] = None,
        port: Optional[str] = None,
        external_connections: "Optional[EIPV]" = None,
    ) -> None:
        super(OutboundConnectionView, self).__init__(dgraph_client, node_key, uid)

        self.dgraph_client = dgraph_client
        self.node_key = node_key
        self.uid = uid
        self.port = port

        self.external_connections = external_connections

    def get_property_tuples(self) -> List[Tuple[str, Any]]:
        props = (
            ("node_key", self.node_key),
            ("uid", self.uid),
            ("port", self.port),
        )
        return [p for p in props if p[1]]

    def get_edge_tuples(self) -> List[Tuple[str, Any]]:
        edges = (
            ('external_connections', self.external_connections)
        )
        return [e for e in edges if e[1]]

    @staticmethod
    def get_property_types() -> List[Tuple[str, Callable[[Any], Union[str, int]]]]:
        return [("port", str)]

    @staticmethod
    def get_edge_types() -> List[Tuple[str, Union[List[Type[V]], Type[V]]]]:
        return [("external_connections", [ExternalIpView])]


class ExternalIpQuery(Queryable):
    def __init__(self) -> None:
        super(ExternalIpQuery, self).__init__(ExternalIpView)
        self._external_ip = []  # type: List[List[Cmp]]

        # Edges
        self._connections_from = None  # type: 'Optional[OCQ]'

        # Meta
        self._first = None  # type: Optional[int]

    def get_properties(self) -> List[Tuple[str, PropertyFilter]]:
        properties = [
            ("node_key", self.get_node_key_filter()),
            ("uid", self.get_uid_filter()),
            ("external_ip", self._external_ip),
        ]

        return properties

    def get_unique_predicate(self) -> Optional[str]:
        return "external_ip"

    def get_node_type_name(self) -> Optional[str]:
        return None

    def get_node_key_filter(self) -> PropertyFilter:
        return [[self._node_key]]

    def get_uid_filter(self) -> PropertyFilter:
        if isinstance(self._uid, Eq):
            return [[self._uid]]
        return []

    def get_forward_edges(self) -> List[Tuple[str, Any]]:
        return []

    def get_reverse_edges(self) -> List[Tuple[str, Any]]:
        edges = [("connections_from", self._connections_from)]
        return [e for e in edges if e[1]]

    def with_external_ip(
        self,
        eq: Optional[Union[str, List[str], Not, List[Not]]] = None,
        contains: Optional[Union[str, List[str], Not, List[Not]]] = None,
        ends_with: Optional[Union[str, List[str], Not, List[Not]]] = None,
    ) -> "EIPQ":
        self._external_ip.extend(_str_cmps("external_ip", eq, contains, ends_with))
        return self

    def with_connections_from(self, process: PQ) -> "EIPQ":
        process = deepcopy(process)
        process._created_connection = self
        self._connections_from = process
        return self


class ExternalIpView(Viewable):
    def __init__(
        self,
        dgraph_client: DgraphClient,
        node_key: str,
        uid: str,
        external_ip: Optional[str] = None,
    ) -> None:
        super(ExternalIpView, self).__init__(dgraph_client, node_key, uid)
        self.dgraph_client = dgraph_client
        self.node_key = node_key
        self.uid = uid
        self.external_ip = external_ip

    def get_property_tuples(self) -> List[Tuple[str, Any]]:
        props = (
                ("node_key", self.node_key),
                ("uid", self.uid),
                ("external_ip", self.external_ip),
        )
        return [p for p in props if p[1]]

    def get_edge_tuples(self) -> List[Tuple[str, Any]]:
        return []

    @staticmethod
    def get_property_types() -> List[Tuple[str, Callable[[Any], Union[str, int]]]]:
        return [("external_ip", str)]

    @staticmethod
    def get_edge_types() -> List[Tuple[str, Union[List[Type[V]], Type[V]]]]:
        return []
