from copy import deepcopy
from collections import defaultdict
from typing import Iterator, TypeVar, Set
from typing import Optional, List, Dict, Any, Union
from typing import Tuple

from pydgraph import DgraphClient

from grapl_analyzerlib.querying import PropertyFilter, StrCmp, IntCmp, EdgeFilter, Has, Cmp, _str_cmps, _int_cmps, \
    Queryable, Not, Eq
from grapl_analyzerlib.querying import Viewable
from grapl_analyzerlib.querying import flatten_nodes

PV = TypeVar("PV", bound='process_node.ProcessView')
PQ = TypeVar("PQ", bound='process_node.ProcessQuery')

FV = TypeVar("FV", bound='file_node.FileView')
FQ = TypeVar("FQ", bound='file_node.FileQuery')

OCV = TypeVar("OCV", bound='outbound_connection_node.OutboundConnectionView')
OCQ = TypeVar("OCQ", bound='outbound_connection_node.OutboundConnectionQuery')

EIPV = TypeVar("EIPV", bound='external_ip_node.ExternalIpView')
EIPQ = TypeVar("EIPQ", bound='external_ip_node.ExternalIpQuery')

N = TypeVar("N", bound='entities.NodeView')
S = TypeVar("S", bound='entities.SubgraphView')

DNQ = TypeVar('DNQ', bound='dynamic_node.DynamicNodeQuery')
DNV = TypeVar('DNV', bound='dynamic_node.DynamicNodeView')

PluginNodeView = TypeVar('PluginNodeView')


class EdgeView(object):
    def __init__(
            self, from_neighbor_key: str, to_neighbor_key: str, edge_name: str
    ) -> None:
        self.from_neighbor_key = from_neighbor_key
        self.to_neighbor_key = to_neighbor_key
        self.edge_name = edge_name


class NodeView(object):
    def __init__(self, node: Union['PV', 'FV', 'EIPV', 'OCV', 'DNV']):
        self.node = node

    @staticmethod
    def from_raw(dgraph_client: DgraphClient, node: Any) -> 'N':
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

    def as_process_view(self) -> Optional['PV']:
        if isinstance(self.node, process_node.ProcessView):
            return self.node
        return None

    def as_file_view(self) -> Optional['FV']:
        if isinstance(self.node, file_node.FileView):
            return self.node
        return None

    def as_dynamic_node(self) -> Optional['DNV']:
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
    def from_proto(dgraph_client: DgraphClient, s: bytes) -> 'S':
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

    def process_iter(self) -> Iterator['PV']:
        for node in self.nodes.values():
            maybe_node = node.as_process_view()
            if maybe_node:
                yield maybe_node

    def file_iter(self) -> Iterator['FV']:
        for node in self.nodes.values():
            maybe_node = node.as_file_view()
            if maybe_node:
                yield maybe_node

class DynamicNodeQuery(Queryable):
    def __init__(self, node_type: str) -> None:
        self.node_type = node_type
        self._node_key = Has(
            "node_key"
        )  # type: Cmp
        self._uid = Has(
            "uid"
        )  # type: Cmp

        # Dict of property name to its associated filters
        self.property_filters = defaultdict(list)  # type: Dict[str, PropertyFilter]

        # Dict of edge name to associated filters
        self.edge_filters = dict()
        self.reverse_edge_filters = dict()

    def with_property_str_filter(self, prop_name: str, eq=StrCmp, contains=StrCmp, ends_with=StrCmp) -> 'DNQ':
        self.property_filters[prop_name].extend(
            _str_cmps(prop_name, eq, contains, ends_with)
        )
        return self

    def with_property_int_filter(self, prop_name: str, eq=IntCmp, contains=IntCmp, ends_with=IntCmp) -> 'DNQ':
        self.property_filters[prop_name].extend(
            _int_cmps(prop_name, eq, contains, ends_with)
        )
        return self

    def with_edge_filter(self, edge: str, edge_filter: EdgeFilter) -> 'DNQ':
        self.edge_filters[edge] = edge_filter
        return self

    def with_reverse_edge_filter(self, edge: str, edge_filter: EdgeFilter) -> 'DNQ':
        self.reverse_edge_filters[edge] = edge_filter
        return self

    # Querable Interface Implementation
    def get_node_type_name(self) -> str:
        return self.node_type

    def get_node_key_filter(self) -> PropertyFilter:
        return [[self._node_key]]

    def get_uid_filter(self) -> PropertyFilter:
        return [[self._uid]]

    def get_properties(self) -> List[Tuple[str, PropertyFilter]]:
        properties = [(p, f) for (p, f) in self.property_filters.items()]
        properties.append(('node_key', self.get_node_key_filter()))
        properties.append(('uid', self.get_uid_filter()))
        return properties

    def get_forward_edges(self) -> List[Tuple[str, Any]]:
        return [(e, f) for (e, f) in self.edge_filters.items()]

    def get_reverse_edges(self) -> List[Tuple[str, Any]]:
        return [(e, f) for (e, f) in self.reverse_edge_filters.items()]


class DynamicNodeView(Viewable):
    def __init__(
            self,
            dgraph_client: DgraphClient,
            node_key: str,
            node_type: str,
            asset_id: Optional[str] = None,
    ):
        self.dgraph_client = dgraph_client
        self.node_key = node_key
        self.node_type = node_type
        self.asset_id = asset_id


class FileQuery(Queryable):
    def __init__(self) -> None:
        # Attributes
        self._node_key = Has(
            "node_key"
        )  # type: Cmp

        self._uid = Has(
            "uid"
        )  # type: Cmp

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
        self._writers = None  # type: Optional[ 'Q']
        self._readers = None  # type: Optional['PQ']
        self._spawned_from = None  # type: Optional['PQ']

    def with_node_key(self, node_key: Optional[str] = None):
        if node_key:
            self._node_key = Eq("node_key", node_key)
        else:
            self._node_key = Has("node_key")
        return self

    def with_file_name(self, eq=None, contains=None, ends_with=None) ->  'FQ':
        self._file_name.extend(
            _str_cmps("file_name", eq, contains, ends_with)
        )
        return self

    def with_asset_id(self, eq=None, contains=None, ends_with=None) ->  'FQ':
        self._asset_id.extend(
            _str_cmps("asset_id", eq, contains, ends_with)
        )
        return self

    def with_file_path(self, eq=None, contains=None, ends_with=None) ->  'FQ':
        self._file_path.extend(
            _str_cmps("file_path", eq, contains, ends_with)
        )
        return self

    def with_file_extension(self, eq=None, contains=None, ends_with=None) ->  'FQ':
        self._file_extension.extend(
            _str_cmps("file_extension", eq, contains, ends_with)
        )
        return self

    def with_file_mime_type(self, eq=None, contains=None, ends_with=None) ->  'FQ':
        self._file_mime_type.extend(
            _str_cmps("file_mime_type", eq, contains, ends_with)
        )
        return self

    def with_file_size(self, eq=None, contains=None, ends_with=None) ->  'FQ':
        self._file_size.extend(
            _str_cmps("file_size", eq, contains, ends_with)
        )
        return self

    def with_file_version(self, eq=None, contains=None, ends_with=None) ->  'FQ':
        self._file_version.extend(
            _str_cmps("file_version", eq, contains, ends_with)
        )
        return self

    def with_file_description(self, eq=None, contains=None, ends_with=None) ->  'FQ':
        self._file_description.extend(
            _str_cmps("file_description", eq, contains, ends_with)
        )
        return self

    def with_file_product(self, eq=None, contains=None, ends_with=None) ->  'FQ':
        self._file_product.extend(
            _str_cmps("file_product", eq, contains, ends_with)
        )
        return self

    def with_file_company(self, eq=None, contains=None, ends_with=None) ->  'FQ':
        self._file_company.extend(
            _str_cmps("file_company", eq, contains, ends_with)
        )
        return self

    def with_file_directory(self, eq=None, contains=None, ends_with=None) ->  'FQ':
        self._file_directory.extend(
            _str_cmps("file_directory", eq, contains, ends_with)
        )
        return self

    def with_file_inode(self, eq=None, contains=None, ends_with=None) ->  'FQ':
        self._file_inode.extend(
            _str_cmps("file_inode", eq, contains, ends_with)
        )
        return self

    def with_file_hard_links(self, eq=None, contains=None, ends_with=None) ->  'FQ':
        self._file_hard_links.extend(
            _str_cmps("file_hard_links", eq, contains, ends_with)
        )
        return self

    def with_md5_hash(self, eq=None, contains=None, ends_with=None) ->  'FQ':
        self._md5_hash.extend(
            _str_cmps("md5_hash", eq, contains, ends_with)
        )
        return self

    def with_sha1_hash(self, eq=None, contains=None, ends_with=None) ->  'FQ':
        self._sha1_hash.extend(
            _str_cmps("sha1_hash", eq, contains, ends_with)
        )
        return self

    def with_sha256_hash(self, eq=None, contains=None, ends_with=None) ->  'FQ':
        self._sha256_hash.extend(
            _str_cmps("sha256_hash", eq, contains, ends_with)
        )
        return self

    def with_creator(self, creator: 'PQ') ->  'FQ':
        creator = deepcopy(creator)
        self._creator = creator
        return self

    def with_deleter(self, deleter: 'PQ') ->  'FQ':
        deleter = deepcopy(deleter)
        self._deleter = deleter
        deleter._deleted_files = self
        return self

    def with_writers(self, writers: 'PQ') ->  'FQ':
        writers = deepcopy(writers)
        self._writers = writers
        return self

    def with_readers(self, readers: 'PQ') ->  'FQ':
        readers = deepcopy(readers)
        self._readers = readers
        readers._read_files = self
        return self

    def with_spawned_from(self, spawned_from: 'PQ') ->  'FQ':
        spawned_from = deepcopy(spawned_from)
        self._spawned_from = spawned_from
        spawned_from._bin_file = self
        return self

    def get_unique_predicate(self) -> Optional[str]:
        return 'file_path'

    def get_node_type_name(self) -> Optional[str]:
        return None

    def get_node_key_filter(self) -> PropertyFilter:
        return [[self._node_key]]

    def get_uid_filter(self) -> PropertyFilter:
        return [[self._uid]]

    def get_properties(self) -> List[Tuple[str, PropertyFilter]]:
        properties = (
            ("node_key", self.get_node_key_filter()),
            ("uid", self.get_uid_filter()),
            ("file_name", self._file_name) if self._file_name else None,
            ("asset_id", self._asset_id) if self._asset_id else None,
            ("file_path", self._file_path) if self._file_path else None,
            ("file_extension", self._file_extension) if self._file_extension else None,
            ("file_mime_type", self._file_mime_type) if self._file_mime_type else None,
            ("file_size", self._file_size) if self._file_size else None,
            ("file_version", self._file_version) if self._file_version else None,
            ("file_description", self._file_description) if self._file_description else None,
            ("file_product", self._file_product) if self._file_product else None,
            ("file_company", self._file_company) if self._file_company else None,
            ("file_directory", self._file_directory) if self._file_directory else None,
            ("file_inode", self._file_inode) if self._file_inode else None,
            ("file_hard_links", self._file_hard_links) if self._file_hard_links else None,
            ("md5_hash", self._md5_hash) if self._md5_hash else None,
            ("sha1_hash", self._sha1_hash) if self._sha1_hash else None,
            ("sha256_hash", self._sha256_hash) if self._sha256_hash else None,
        )

        return [p for p in properties if p]

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

    def get_reverse_edges(self) -> List[Tuple[str, Any]]:
        neighbors = (
            ("~created_file", self._creator) if self._creator else None,
            ("~deleted_file", self._deleter) if self._deleter else None,
            ("~wrote_to_files", self._writers) if self._writers else None,
            ("~read_files", self._readers) if self._readers else None,
            ("~bin_file", self._spawned_from) if self._spawned_from else None,
        )

        return [n for n in neighbors if n]

    def query_first(self, dgraph_client, contains_node_key=None) -> Optional['FV']:
        return super(FileQuery, self)._query_first(dgraph_client, FileView, contains_node_key)


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

            creator: Optional[List['PV']] = None,
            deleter: Optional[List['PV']] = None,
            writers: Optional[List['PV']] = None,
            readers: Optional[List['PV']] = None,
            spawned_from: Optional[List['PV']] = None,
    ) -> None:
        super(FileView, self).__init__(self)
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

    @staticmethod
    def from_dict(dgraph_client: DgraphClient, d: Dict[str, Any]) -> 'FV':

        raw_creator = d.get("~created_file", None)
        raw_deleter = d.get("~deleted_file", None)
        raw_writers = d.get("~wrote_to_files", None)
        raw_readers = d.get("~read_files", None)
        raw_spawned_from = d.get("~bin_file", None)

        creator = None  # type: Optional[List['PV']]
        if raw_creator:
            creator = process_node.ProcessView.from_dict(dgraph_client, raw_creator)

        deleter = None  # type: Optional[List['PV']]
        if raw_deleter:
            deleter = process_node.ProcessView.from_dict(dgraph_client, raw_deleter)

        writers = None  # type: Optional[List['PV']]
        if raw_writers:
            writers = [
                process_node.ProcessView.from_dict(dgraph_client, raw) for raw in raw_writers
            ]

        readers = None  # type: Optional[List['PV']]
        if raw_readers:
            readers = [
                process_node.ProcessView.from_dict(dgraph_client, raw) for raw in raw_readers
            ]

        spawned_from = None  # type: Optional[List['PV']]
        if raw_spawned_from:
            spawned_from = [
                process_node.ProcessView.from_dict(dgraph_client, raw) for raw in raw_spawned_from
            ]

        return FileView(
            dgraph_client=dgraph_client,
            node_key=d["node_key"],
            uid=d["uid"],
            asset_id=d.get("asset_id"),
            file_path=d.get("file_path"),
            file_name=d.get("file_name"),
            file_extension=d.get("file_extension"),
            file_mime_type=d.get("file_mime_type"),
            file_size=d.get("file_size"),
            file_version=d.get("file_version"),
            file_description=d.get("file_description"),
            file_product=d.get("file_product"),
            file_company=d.get("file_company"),
            file_directory=d.get("file_directory"),
            file_inode=d.get("file_inode"),
            file_hard_links=d.get("file_hard_links"),
            md5_hash=d.get("md5_hash"),
            sha1_hash=d.get("sha1_hash"),
            sha256_hash=d.get("sha256_hash"),

            creator=creator,
            deleter=deleter,
            writers=writers,
            readers=readers,
            spawned_from=spawned_from,
        )

    def get_file_name(self) -> Optional[str]:
        if self.file_name:
            return self.file_name

        self_file = (
            FileQuery()
                .with_node_key(self.node_key)
                .with_file_name()
                .query_first(dgraph_client=self.dgraph_client)
        )

        if not self_file:
            return None

        self.file_name = self_file[0].file_name
        return self.file_name

    def get_file_extension(self) -> Optional[str]:
        if self.file_extension:
            return self.file_extension

        self_file = (
            FileQuery()
                .with_node_key(self.node_key)
                .with_file_extension()
                .query_first(dgraph_client=self.dgraph_client)
        )

        if not self_file:
            return None

        self.file_extension = self_file[0].file_extension
        return self.file_extension

    def get_file_mime_type(self) -> Optional[str]:
        if self.file_mime_type:
            return self.file_mime_type

        self_file = (
            FileQuery()
                .with_node_key(self.node_key)
                .with_file_mime_type()
                .query_first(dgraph_client=self.dgraph_client)
        )

        if not self_file:
            return None

        self.file_mime_type = self_file[0].file_mime_type
        return self.file_mime_type

    def get_file_size(self) -> Optional[int]:
        if self.file_size:
            return self.file_size

        self_file = (
            FileQuery()
                .with_node_key(self.node_key)
                .with_file_size()
                .query_first(dgraph_client=self.dgraph_client)
        )

        if not self_file:
            return None

        if not self_file[0].file_size:
            return None

        self.file_size = int(self_file[0].file_size)
        return self.file_size

    def get_file_version(self) -> Optional[str]:
        if self.file_version:
            return self.file_version

        self_file = (
            FileQuery()
                .with_node_key(self.node_key)
                .with_file_version()
                .query_first(dgraph_client=self.dgraph_client)
        )

        if not self_file:
            return None

        self.file_version = self_file[0].file_version
        return self.file_version

    def get_file_description(self) -> Optional[str]:
        if self.file_description:
            return self.file_description

        self_file = (
            FileQuery()
                .with_node_key(self.node_key)
                .with_file_description()
                .query_first(dgraph_client=self.dgraph_client)
        )

        if not self_file:
            return None

        self.file_description = self_file[0].file_description
        return self.file_description

    def get_file_product(self) -> Optional[str]:
        if self.file_product:
            return self.file_product

        self_file = (
            FileQuery()
                .with_node_key(self.node_key)
                .with_file_product()
                .query_first(dgraph_client=self.dgraph_client)
        )

        if not self_file:
            return None

        self.file_product = self_file[0].file_product
        return self.file_product

    def get_file_company(self) -> Optional[str]:
        if self.file_company:
            return self.file_company

        self_file = (
            FileQuery()
                .with_node_key(self.node_key)
                .with_file_company()
                .query_first(dgraph_client=self.dgraph_client)
        )

        if not self_file:
            return None

        self.file_company = self_file[0].file_company
        return self.file_company

    def get_file_directory(self) -> Optional[str]:
        if self.file_directory:
            return self.file_directory

        self_file = (
            FileQuery()
                .with_node_key(self.node_key)
                .with_file_directory()
                .query_first(dgraph_client=self.dgraph_client)
        )

        if not self_file:
            return None

        self.file_directory = self_file[0].file_directory
        return self.file_directory

    def get_file_inode(self) -> Optional[str]:
        if self.file_inode:
            return self.file_inode

        self_file = (
            FileQuery()
                .with_node_key(self.node_key)
                .with_file_inode()
                .query_first(dgraph_client=self.dgraph_client)
        )

        if not self_file:
            return None

        self.file_inode = self_file[0].file_inode
        return self.file_inode

    def get_file_hard_links(self) -> Optional[str]:
        if self.file_hard_links:
            return self.file_hard_links

        self_file = (
            FileQuery()
                .with_node_key(self.node_key)
                .with_file_hard_links()
                .query_first(dgraph_client=self.dgraph_client)
        )

        if not self_file:
            return None

        self.file_hard_links = self_file[0].file_hard_links
        return self.file_hard_links

    def get_md5_hash(self) -> Optional[str]:
        if self.md5_hash:
            return self.md5_hash

        self_file = (
            FileQuery()
                .with_node_key(self.node_key)
                .with_md5_hash()
                .query_first(dgraph_client=self.dgraph_client)
        )

        if not self_file:
            return None

        self.md5_hash = self_file[0].md5_hash
        return self.md5_hash

    def get_sha1_hash(self) -> Optional[str]:
        if self.sha1_hash:
            return self.sha1_hash

        self_file = (
            FileQuery()
                .with_node_key(self.node_key)
                .with_sha1_hash()
                .query_first(dgraph_client=self.dgraph_client)
        )

        if not self_file:
            return None

        self.sha1_hash = self_file[0].sha1_hash
        return self.sha1_hash

    def get_sha256_hash(self) -> Optional[str]:
        if self.sha256_hash:
            return self.sha256_hash

        self_file = (
            FileQuery()
                .with_node_key(self.node_key)
                .with_sha256_hash()
                .query_first(dgraph_client=self.dgraph_client)
        )

        if not self_file:
            return None

        self.sha256_hash = self_file[0].sha256_hash
        return self.sha256_hash

    def get_file_path(self) -> Optional[str]:
        if self.file_path:
            return self.file_path

        self_file = (
            FileQuery()
                .with_node_key(self.node_key)
                .with_file_path()
                .query_first(dgraph_client=self.dgraph_client)
        )

        if not self_file:
            return None

        self.file_path = self_file[0].file_path
        return self.file_path

    def get_asset_id(self) -> Optional[str]:
        if self.asset_id:
            return self.asset_id

        self_file = (
            FileQuery()
                .with_node_key(self.node_key)
                .with_asset_id()
                .query_first(dgraph_client=self.dgraph_client)
        )

        if not self_file:
            return None

        self.asset_id = self_file[0].asset_id
        return self.asset_id

    def to_dict(self, root=False) -> Dict[str, Any]:
        node_dict = dict()

        if self.node_key:
            node_dict['node_key'] = self.node_key

        if self.uid:
            node_dict['uid'] = self.uid

        if self.asset_id:
            node_dict['asset_id'] = self.asset_id

        if self.file_name:
            node_dict['file_name'] = self.file_name

        if self.file_path:
            node_dict['file_path'] = self.file_path

        if self.file_extension:
            node_dict['file_extension'] = self.file_extension

        if self.file_mime_type:
            node_dict['file_mime_type'] = self.file_mime_type

        if self.file_size:
            node_dict['file_size'] = self.file_size

        if self.file_version:
            node_dict['file_version'] = self.file_version

        if self.file_description:
            node_dict['file_description'] = self.file_description

        if self.file_product:
            node_dict['file_product'] = self.file_product

        if self.file_company:
            node_dict['file_company'] = self.file_company

        if self.file_directory:
            node_dict['file_directory'] = self.file_directory

        if self.file_inode:
            node_dict['file_inode'] = self.file_inode

        if self.file_hard_links:
            node_dict['file_hard_links'] = self.file_hard_links

        if self.md5_hash:
            node_dict['md5_hash'] = self.md5_hash

        if self.sha1_hash:
            node_dict['sha1_hash'] = self.sha1_hash

        if self.sha256_hash:
            node_dict['sha256_hash'] = self.sha256_hash

        if root:
            node_dict['root'] = True

        return {'node': node_dict, 'edges': []}  # TODO: Generate edges



class ProcessQuery(Queryable):
    def __init__(self) -> None:
        super(ProcessQuery, self).__init__()
        # Properties
        self._node_key = Has(
            "node_key"
        )  # type: Cmp
        self._uid = Has(
            "uid"
        )  # type: Cmp

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
        self._created_connection = None  # type: Optional[ 'OCQ']

        self._parent = None  # type: Optional[PQ]

        # Meta
        self._first = None  # type: Optional[int]

    def with_node_key(self, node_key: Optional[Union[Not, str]] = None):
        if node_key:
            self._node_key = Eq("node_key", node_key)
        else:
            self._node_key = Has("node_key")
        return self

    def with_uid(self, uid: Optional[Union[Not, str]] = None):
        if uid:
            self._uid = Eq("uid", uid)
        else:
            self._uid = Has("uid")
        return self

    def only_first(self, first: int) -> 'PQ':
        self._first = first
        return self

    def with_asset_id(
            self,
            eq: Optional[
                Union[str, List[str], Not, List[Not]]
            ] = None,
            contains: Optional[
                Union[str, List[str], Not, List[Not]]
            ] = None,
            ends_with: Optional[
                Union[str, List[str], Not, List[Not]]
            ] = None,
    ) -> 'PQ':
        self._asset_id.extend(
            _str_cmps("asset_id", eq, contains, ends_with)
        )
        return self

    def with_process_name(
            self,
            eq: Optional[
                Union[str, List[str], Not, List[Not]]
            ] = None,
            contains: Optional[
                Union[str, List[str], Not, List[Not]]
            ] = None,
            ends_with: Optional[
                Union[str, List[str], Not, List[Not]]
            ] = None,
    ) -> 'PQ':
        self._process_name.extend(
            _str_cmps("process_name", eq, contains, ends_with)
        )
        return self

    def with_process_command_line(
            self,
            eq: Optional[
                Union[str, List[str], Not, List[Not]]
            ] = None,
            contains: Optional[
                Union[str, List[str], Not, List[Not]]
            ] = None,
            ends_with: Optional[
                Union[str, List[str], Not, List[Not]]
            ] = None,
    ) -> 'PQ':
        self._process_command_line.extend(
            _str_cmps("process_command_line", eq, contains, ends_with)
        )
        return self

    def with_process_guid(
            self,
            eq: Optional[
                Union[str, List[str], Not, List[Not]]
            ] = None,
            contains: Optional[
                Union[str, List[str], Not, List[Not]]
            ] = None,
            ends_with: Optional[
                Union[str, List[str], Not, List[Not]]
            ] = None,
    ) -> 'PQ':
        self._process_guid.extend(
            _str_cmps("process_guid", eq, contains, ends_with)
        )
        return self

    def with_process_id(
            self,
            eq: Optional[
                Union[str, List[int], Not, List[Not]]
            ] = None,
            gt: Optional[
                Union[str, List[int], Not, List[Not]]
            ] = None,
            lt: Optional[
                Union[str, List[int], Not, List[Not]]
            ] = None,
    ) -> 'PQ':
        self._process_id.extend(_int_cmps("process_id", eq, gt, lt))
        return self

    def with_created_timestamp(
            self,
            eq: Optional[
                Union[str, List[int], Not, List[Not]]
            ] = None,
            gt: Optional[
                Union[str, List[int], Not, List[Not]]
            ] = None,
            lt: Optional[
                Union[str, List[int], Not, List[Not]]
            ] = None,
    ) -> 'PQ':
        self._created_timestamp.extend(
            _int_cmps("created_timestamp", eq, gt, lt)
        )
        return self

    def with_terminated_timestamp(
            self,
            eq: Optional[
                Union[str, List[int], Not, List[Not]]
            ] = None,
            gt: Optional[
                Union[str, List[int], Not, List[Not]]
            ] = None,
            lt: Optional[
                Union[str, List[int], Not, List[Not]]
            ] = None,
    ) -> 'PQ':
        self._terminated_timestamp.extend(
            _int_cmps("terminated_timestamp", eq, gt, lt)
        )
        return self

    def with_last_seen_timestamp(
            self,
            eq: Optional[
                Union[str, List[int], Not, List[Not]]
            ] = None,
            gt: Optional[
                Union[str, List[int], Not, List[Not]]
            ] = None,
            lt: Optional[
                Union[str, List[int], Not, List[Not]]
            ] = None,
    ) -> 'PQ':
        self._last_seen_timestamp.extend(
            _int_cmps("last_seen_timestamp", eq, gt, lt)
        )
        return self

    def with_parent(self, process: 'PQ') -> 'PQ':
        process: 'PQ' = deepcopy(process)
        process._children = self
        self._parent = process
        return self

    def with_bin_file(self, file: 'FQ') -> 'PQ':
        file = deepcopy(file)
        file._spawned_from = self
        self._bin_file = file
        return self

    def with_deleted_files(self, file: 'FQ') -> 'PQ':
        file = deepcopy(file)
        file._deleter = self
        self._deleted_files = file
        return self

    def with_created_files(self, file: 'FQ') -> 'PQ':
        file = deepcopy(file)
        file._creator = self
        self._created_files = file
        return self

    def with_written_files(self, file: 'FQ') -> 'PQ':
        file = deepcopy(file)
        file._writers = self
        self._wrote_to_files = file
        return self

    def with_read_files(self, file: 'FQ') -> 'PQ':
        file = deepcopy(file)
        file._readers = self
        self._read_files = file
        return self

    def with_children(self, children: 'PQ') -> 'PQ':
        children = deepcopy(children)
        children._parent = self
        self._children = children
        return self

    def with_created_connection(
            self,
            outbound_conn: Union[ 'OCQ', 'EIPQ']
    ) -> 'PQ':
        outbound_conn = deepcopy(outbound_conn)

        if isinstance(outbound_conn, external_ip_node.ExternalIpQuery):
            outbound_conn = (
                outbound_connection_node.OutboundConnectionQuery()
                    .with_external_connection(outbound_conn)
            )
        outbound_conn._connecting_process = self

        self._created_connection = outbound_conn
        return self

    def query_first(self, dgraph_client, contains_node_key=None) -> Optional['PV']:
        return super(ProcessQuery, self)._query_first(
            dgraph_client, ProcessView, contains_node_key
        )

    def get_properties(self) -> List[Tuple[str, PropertyFilter]]:
        properties = []
        if self._asset_id:
            properties.append(("asset_id", self._asset_id))
        if self._process_name:
            properties.append(("process_name", self._process_name))
        if self._process_command_line:
            properties.append(("process_command_line", self._process_command_line))
        if self._process_guid:
            properties.append(("process_guid", self._process_guid))
        if self._process_id:
            properties.append(("process_id", self._process_id))
        if self._created_timestamp:
            properties.append(("created_timestamp", self._created_timestamp))
        if self._terminated_timestamp:
            properties.append(("terminated_timestamp", self._terminated_timestamp))
        if self._last_seen_timestamp:
            properties.append(("last_seen_timestamp", self._last_seen_timestamp))

        properties.append(('node_key', self.get_node_key_filter()))
        properties.append(('uid', self.get_uid_filter()))

        return properties

    def get_node_type_name(self) -> Optional[str]:
        return None

    def get_unique_predicate(self) -> Optional[str]:
        return 'process_id'

    def get_node_key_filter(self) -> PropertyFilter:
        return [[self._node_key]]

    def get_uid_filter(self) -> PropertyFilter:
        return [[self._uid]]

    def get_forward_edges(self) -> List[Tuple[str, Any]]:
        edges = [
            ("bin_file", self._bin_file) if self._bin_file else None,
            ("children", self._children) if self._children else None,
            ("deleted_files", self._deleted_files) if self._deleted_files else None,
            ("created_files", self._created_files) if self._created_files else None,
            ("wrote_to_files", self._wrote_to_files) if self._wrote_to_files else None,
            ("read_files", self._read_files) if self._read_files else None,
            ("created_connection", self._created_connection) if self._created_connection else None,
        ]

        return [e for e in edges if e]

    def get_reverse_edges(self) -> List[Tuple[str, Any]]:
        edges = [
            ("~children", self._parent) if self._parent else None,
        ]

        return [e for e in edges if e]



class ProcessView(Viewable):
    def __init__(
            self,
            dgraph_client: DgraphClient,
            node_key: str,
            uid: Optional[str] = None,
            asset_id: Optional[str] = None,
            process_name: Optional[str] = None,
            process_command_line: Optional[str] = None,
            process_guid: Optional[str] = None,
            process_id: Optional[str] = None,
            created_timestamp: Optional[str] = None,
            terminated_timestamp: Optional[str] = None,
            last_seen_timestamp: Optional[str] = None,
            bin_file: Optional['FV'] = None,
            parent: Optional['PV'] = None,
            children: Optional[List['PV']] = None,
            deleted_files: Optional[List['FV']] = None,
            created_files: Optional[List['FV']] = None,
            read_files: Optional[List['FV']] = None,
            created_connections: Optional[List['EIPV']] = None,
    ) -> None:
        super(ProcessView, self).__init__(self)

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

        # TODO: Support created, deleted, written, read
        self.bin_file = bin_file  # type: Optional['FV']
        self.children = children  # type: Optional[List['PV']]
        self.parent = parent  # type: Optional['PV']
        self.deleted_files = deleted_files  # type: Optional[List['FV']]
        self.created_files = created_files  # type: Optional[List['FV']]
        self.read_files = read_files  # type: Optional[List['FV']]
        self.created_connections = created_connections  # type: Optional[List['EIPV']]

    @staticmethod
    def from_dict(dgraph_client: DgraphClient, d: Dict[str, Any]) -> 'PV':
        raw_created_connections = d.get("created_connection", None)

        created_connections = None

        if raw_created_connections:
            created_connections = [
                outbound_connection_node.OutboundConnectionView.from_dict(dgraph_client, c)
                for c in raw_created_connections
            ]


        raw_bin_file = d.get("bin_file", None)

        bin_file = None

        if raw_bin_file:
            bin_file = file_node.FileView.from_dict(dgraph_client, raw_bin_file[0])

        raw_parent = d.get("~children", None)

        parent = None

        if raw_parent:
            parent = ProcessView.from_dict(dgraph_client, raw_parent[0])

        raw_deleted_files = d.get("deleted_files", None)

        deleted_files = None

        if raw_deleted_files:
            deleted_files = [
                file_node.FileView.from_dict(dgraph_client, f) for f in d["deleted_files"]
            ]

        raw_read_files = d.get("read_files", None)

        read_files = None

        if raw_read_files:
            read_files = [
                file_node.FileView.from_dict(dgraph_client, f) for f in d["read_files"]
            ]

        raw_created_files = d.get("created_files", None)

        created_files = None

        if raw_created_files:
            created_files = [
                file_node.FileView.from_dict(dgraph_client, f) for f in d["created_files"]
            ]

        raw_children = d.get("children", None)

        children = None  # type: Optional[List['PV']]
        if raw_children:
            children = [
                ProcessView.from_dict(dgraph_client, child) for child in d["children"]
            ]

        return ProcessView(
            dgraph_client=dgraph_client,
            node_key=d["node_key"],
            uid=d["uid"],
            asset_id=d.get("asset_id"),
            process_name=d.get("process_name"),
            process_command_line=d.get("process_command_line"),
            process_guid=d.get("process_guid"),
            process_id=d.get("process_id"),
            created_timestamp=d.get("created_timestamp"),
            terminated_timestamp=d.get("terminated_timestamp"),
            last_seen_timestamp=d.get("last_seen_timestamp"),
            bin_file=bin_file,
            deleted_files=deleted_files,
            read_files=read_files,
            created_files=created_files,
            children=children,
            parent=parent,
            created_connections=created_connections,
        )

    def get_asset_id(self) -> Optional[str]:
        if self.asset_id:
            return self.asset_id

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

    def get_process_name(self) -> Optional[str]:
        if self.process_name:
            return self.process_name

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

    def get_process_command_line(self) -> Optional[str]:
        if self.process_command_line:
            return self.process_command_line

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

    def get_parent(self) -> Optional['PV']:
        if self.parent:
            return self.parent

        parent = (
            ProcessQuery()
                .with_children(ProcessQuery().with_node_key(self.node_key))
                .query_first(self.dgraph_client)
        )

        if not parent:
            return None

        self.parent = parent
        return self.parent

    def get_children(self) -> Optional[List['PV']]:
        if self.children:
            return self.children

        self_node = (
            ProcessQuery()
                .with_node_key(self.node_key)
                .with_children(ProcessQuery().with_node_key())
                .query_first(self.dgraph_client)
        )  # type: 'PV'

        self.children = self_node.children or None

        return self.children

    def get_uid(self):
        # type: () -> str
        if self.uid:
            return self.uid

        process = (
            ProcessQuery()
                .with_node_key(self.node_key)
                .with_uid()
                .query_first(self.dgraph_client)
        )

        assert process
        self.uid = process.uid
        return process.uid

    def get_bin_file(self) -> Optional['FV']:
        if self.bin_file:
            return self.bin_file

        query = (
            ProcessQuery()
                .with_node_key(self.node_key)
                .with_bin_file(file_node.FileQuery())
                .to_query()
        )

        res = json.loads(self.dgraph_client.txn(read_only=True).query(query).json)

        bin_file = res["q0"]["bin_file"]
        self.bin_file = file_node.FileView.from_dict(self.dgraph_client, bin_file[0])
        return self.bin_file

    def get_deleted_files(self) -> Optional[List['FV']]:
        if self.deleted_files:
            return self.deleted_files

        deleted_files = (
            ProcessQuery()
                .with_node_key(self.node_key)
                .with_deleted_files(file_node.FileQuery().with_node_key())
                .query()
        )

        if not deleted_files:
            return None

        self.deleted_files = deleted_files[0].deleted_files
        return self.deleted_files

    def get_read_files(self) -> Optional[List['FV']]:
        if self.read_files:
            return self.read_files

        read_files = (
            ProcessQuery()
                .with_node_key(self.node_key)
                .with_read_files(file_node.FileQuery().with_node_key())
                .query()
        )

        if not read_files:
            return None

        self.read_files = read_files[0].read_files
        return self.read_files

    def get_neighbors(self) -> List[Any]:
        neighbors = (self.parent, self.bin_file, self.children, self.deleted_files)

        return [n for n in neighbors if n]

    def get_edges(self) -> List[Tuple[str, Any]]:
        neighbors = (
            ("parent", self.parent) if self.parent else None,
            ("bin_file", self.bin_file) if self.bin_file else None,
            ("children", self.children) if self.children else None,
            ("deleted_files", self.deleted_files) if self.deleted_files else None,
        )

        return [n for n in neighbors if n]

    def to_dict(self, root=False) -> Dict[str, Any]:
        node_dict = dict()
        edges = []
        node_dict['node_type'] = 'Process'
        if self.node_key:
            node_dict['node_key'] = self.node_key
        if self.uid:
            node_dict['uid'] = self.uid
        if self.process_command_line:
            node_dict['process_command_line'] = self.process_command_line
        if self.process_guid:
            node_dict['process_guid'] = self.process_guid
        if self.process_id:
            node_dict['process_id'] = self.process_id
        if self.created_timestamp:
            node_dict['created_timestamp'] = self.created_timestamp
        if self.terminated_timestamp:
            node_dict['terminated_timestamp'] = self.terminated_timestamp
        if self.last_seen_timestamp:
            node_dict['last_seen_timestamp'] = self.last_seen_timestamp
        if self.process_name:
            node_dict['process_name'] = self.process_name

        if self.asset_id:
            node_dict['process_name'] = self.asset_id

        if self.bin_file:
            node_dict['bin_file'] = self.bin_file.node_key
            edges.append(
                {
                    'from': self.node_key,
                    'edge_name': 'bin_file',
                    'to': self.bin_file.node_key
                }
            )

        if self.children:
            for child in self.children:
                edges.append(
                    {
                        'from': self.node_key,
                        'edge_name': 'children',
                        'to': child.node_key
                    }
                )
        if self.parent:
            edges.append(
                {
                    'from': self.parent.node_key,
                    'edge_name': 'children',
                    'to': self.node_key
                }
            )

        if self.deleted_files:
            for deleted_file in self.deleted_files:
                edges.append(
                    {
                        'from': self.node_key,
                        'edge_name': 'deleted_files',
                        'to': deleted_file.node_key
                    }
                )
        if root:
            node_dict['root'] = True

        return {'node': node_dict, 'edges': edges}




class OutboundConnectionQuery(Queryable):
    def get_unique_predicate(self) -> Optional[str]:
        return 'port'

    def get_node_type_name(self) -> Optional[str]:
        return None

    def get_node_key_filter(self) -> PropertyFilter:
        return [[self._node_key]]

    def get_uid_filter(self) -> PropertyFilter:
        return [[self._uid]]

    def get_properties(self) -> List[Tuple[str, PropertyFilter]]:
        properties = (
            ("node_key", self.get_node_key_filter()),
            ("uid", self.get_uid_filter()),
            ("create_time", self._create_time) if self._create_time else None,
            ("terminate_time", self._terminate_time) if self._terminate_time else None,
            ("last_seen_time", self._last_seen_time) if self._last_seen_time else None,
            ("ip", self._ip) if self._ip else None,
            ("port", self._port) if self._port else None,
        )

        return [p for p in properties if p]

    def get_forward_edges(self) -> List[Tuple[str, Any]]:
        pass

    def get_reverse_edges(self) -> List[Tuple[str, Any]]:
        pass

    def __init__(self) -> None:
        self._node_key = Has(
            "node_key"
        )  # type: Cmp
        self._uid = Has(
            "uid"
        )  # type: Cmp

        self._create_time = []  # type: List[List[Cmp]]
        self._terminate_time = []  # type: List[List[Cmp]]
        self._last_seen_time = []  # type: List[List[Cmp]]
        self._ip = []  # type: List[List[Cmp]]
        self._port = []  # type: List[List[Cmp]]

        # self._internal_connection = None  # type: Optional[Any]
        self._external_connection = None  # type: 'Optional[EIPQ]'
        self._connecting_process = None  # type: 'Optional[PQ]'

    def with_external_connection(
            self,
            external_ip: 'EIPQ'
    ) -> 'OCQ':
        external_ip = deepcopy(external_ip)
        external_ip._connections_from = self
        self._external_connection = external_ip
        return self

    def with_connecting_process(
            self,
            process: 'PQ'
    ) -> 'OCQ':
        process = deepcopy(process)
        process._created_connection = self
        self._connecting_process = process
        return self


class OutboundConnectionView(Viewable):
    def __init__(self,
                 dgraph_client: DgraphClient,
                 node_key: str,
                 uid: Optional[str] = None,
                 port: Optional[str] = None,
                 external_connections: 'Optional[EIPV]' = None,
                 ) -> None:
        super(OutboundConnectionView, self).__init__(self)

        self.dgraph_client = dgraph_client
        self.node_key = node_key
        self.uid = uid
        self.port = port

        self.external_connections = external_connections

    @staticmethod
    def from_dict(dgraph_client: DgraphClient, d: Dict[str, Any]) -> 'OCV':
        raw_external_connection = d.get('external_connection', None)

        external_connection = None  # type: Optional[EIPV]
        if raw_external_connection:
            external_connection = external_ip_node.ExternalIpView.from_dict(dgraph_client, raw_external_connection[0])

        return OutboundConnectionView(
            dgraph_client=dgraph_client,
            node_key=d['node_key'],
            uid=d['uid'],
            port=d.get('port'),
            external_connections=external_connection,
        )



class ExternalIpQuery(Queryable):
    def __init__(self) -> None:
        self._node_key = Has(
            "node_key"
        )  # type: Cmp
        self._uid = Has(
            "uid"
        )  # type: Cmp

        self._external_ip = []  # type: List[List[Cmp]]

        # Edges
        self._connections_from = None  # type: 'Optional[OCQ]/

        # Meta
        self._first = None  # type: Optional[int]

    def with_external_ip(
            self,
            eq: Optional[
                Union[str, List[str], Not, List[Not]]
            ] = None,
            contains: Optional[
                Union[str, List[str], Not, List[Not]]
            ] = None,
            ends_with: Optional[
                Union[str, List[str], Not, List[Not]]
            ] = None,
    ) -> 'EIPQ':
        self._external_ip.extend(
            _str_cmps("external_ip", eq, contains, ends_with)
        )
        return self

    def with_connections_from(
            self,
            process: PQ
    ) -> 'EIPQ':
        process = deepcopy(process)
        process._created_connection = self
        self._connections_from = process
        return self

    def get_properties(self) -> List[str]:
        properties = (
            "node_key" if self._node_key else None,
            "uid" if self._uid else None,
            "external_ip" if self._external_ip else None,
        )

        return [p for p in properties if p]

    def get_neighbors(self) -> List[Any]:
        neighbors = (self._connections_from,)

        return [n for n in neighbors if n]

    def get_edges(self) -> List[Tuple[str, Any]]:
        neighbors = (
            ("connections_from", self._connections_from) if self._connections_from else None,
        )

        return [n for n in neighbors if n]

    def _get_var_block(
            self, binding_num: int, root: Any, already_converted: Set[Any]
    ) -> str:
        if self in already_converted:
            return ""
        already_converted.add(self)

        filters = self._filters()

        connections_from = get_var_block(
            self._connections_from, "~external_connections", binding_num, root, already_converted
        )

        block = f"""
            {filters} {{
                {connections_from}
            }}
            """

        return block

    def _get_var_block_root(
            self, binding_num: int, root: Any, node_key: Optional[str] = None
    ) -> str:
        already_converted = {self}
        root_var = ""
        if self == root:
            root_var = f"Binding{binding_num} as "

        filters = self._filters()

        connections_from = get_var_block(
            self._connections_from, "~external_connections", binding_num, root, already_converted
        )

        func_filter = """has(external_ip)"""
        if node_key:
            func_filter = f'eq(node_key, "{node_key}")'

        block = f"""
            {root_var} var(func: {func_filter}) @cascade {filters} {{
                {connections_from}
            }}
            """

        return block

    def _filters(self) -> str:
        inner_filters = (
            _generate_filter(self._connections_from),
        )

        inner_filters = [i for i in inner_filters if i]
        if not inner_filters:
            return ""
        return f"@filter({'AND'.join(inner_filters)})"



class ExternalIpView(Viewable):
    def __init__(self, dgraph_client: DgraphClient, node_key: str, uid: Optional[str] = None,
                 external_ip: Optional[str] = None) -> None:
        super(ExternalIpView, self).__init__(self)
        self.dgraph_client = dgraph_client
        self.node_key = node_key
        self.uid = uid
        self.external_ip = external_ip

    @staticmethod
    def from_dict(dgraph_client: DgraphClient, d: Dict[str, Any]) -> EIPV:

        return ExternalIpView(
            dgraph_client=dgraph_client,
            node_key=d['node_key'],
            uid=d['uid'],
            external_ip=d.get('external_ip', None),
        )















if __name__ == '__main__':
    ProcessQuery().query_first(
        None, "119895c8-f222-4e0b-906a-b33149ce6de1"
    )