from copy import deepcopy
from typing import List, Optional, Any, Tuple, Dict

from pydgraph import DgraphClient

import grapl_analyzerlib.process_node as process_node
import grapl_analyzerlib.node_types as node_types
from grapl_analyzerlib.querying import Has, Cmp, Queryable, Eq, _str_cmps, Viewable, PropertyFilter


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
        self._creator = None  # type: Optional['node_types.PQ']
        self._deleter = None  # type: Optional['node_types.PQ']
        self._writers = None  # type: Optional[ 'node_types.Q']
        self._readers = None  # type: Optional['node_types.PQ']
        self._spawned_from = None  # type: Optional['node_types.PQ']

    def with_node_key(self, node_key: Optional[str] = None):
        if node_key:
            self._node_key = Eq("node_key", node_key)
        else:
            self._node_key = Has("node_key")
        return self

    def with_file_name(self, eq=None, contains=None, ends_with=None) ->  'node_types.FQ':
        self._file_name.extend(
            _str_cmps("file_name", eq, contains, ends_with)
        )
        return self

    def with_asset_id(self, eq=None, contains=None, ends_with=None) ->  'node_types.FQ':
        self._asset_id.extend(
            _str_cmps("asset_id", eq, contains, ends_with)
        )
        return self

    def with_file_path(self, eq=None, contains=None, ends_with=None) ->  'node_types.FQ':
        self._file_path.extend(
            _str_cmps("file_path", eq, contains, ends_with)
        )
        return self

    def with_file_extension(self, eq=None, contains=None, ends_with=None) ->  'node_types.FQ':
        self._file_extension.extend(
            _str_cmps("file_extension", eq, contains, ends_with)
        )
        return self

    def with_file_mime_type(self, eq=None, contains=None, ends_with=None) ->  'node_types.FQ':
        self._file_mime_type.extend(
            _str_cmps("file_mime_type", eq, contains, ends_with)
        )
        return self

    def with_file_size(self, eq=None, contains=None, ends_with=None) ->  'node_types.FQ':
        self._file_size.extend(
            _str_cmps("file_size", eq, contains, ends_with)
        )
        return self

    def with_file_version(self, eq=None, contains=None, ends_with=None) ->  'node_types.FQ':
        self._file_version.extend(
            _str_cmps("file_version", eq, contains, ends_with)
        )
        return self

    def with_file_description(self, eq=None, contains=None, ends_with=None) ->  'node_types.FQ':
        self._file_description.extend(
            _str_cmps("file_description", eq, contains, ends_with)
        )
        return self

    def with_file_product(self, eq=None, contains=None, ends_with=None) ->  'node_types.FQ':
        self._file_product.extend(
            _str_cmps("file_product", eq, contains, ends_with)
        )
        return self

    def with_file_company(self, eq=None, contains=None, ends_with=None) ->  'node_types.FQ':
        self._file_company.extend(
            _str_cmps("file_company", eq, contains, ends_with)
        )
        return self

    def with_file_directory(self, eq=None, contains=None, ends_with=None) ->  'node_types.FQ':
        self._file_directory.extend(
            _str_cmps("file_directory", eq, contains, ends_with)
        )
        return self

    def with_file_inode(self, eq=None, contains=None, ends_with=None) ->  'node_types.FQ':
        self._file_inode.extend(
            _str_cmps("file_inode", eq, contains, ends_with)
        )
        return self

    def with_file_hard_links(self, eq=None, contains=None, ends_with=None) ->  'node_types.FQ':
        self._file_hard_links.extend(
            _str_cmps("file_hard_links", eq, contains, ends_with)
        )
        return self

    def with_md5_hash(self, eq=None, contains=None, ends_with=None) ->  'node_types.FQ':
        self._md5_hash.extend(
            _str_cmps("md5_hash", eq, contains, ends_with)
        )
        return self

    def with_sha1_hash(self, eq=None, contains=None, ends_with=None) ->  'node_types.FQ':
        self._sha1_hash.extend(
            _str_cmps("sha1_hash", eq, contains, ends_with)
        )
        return self

    def with_sha256_hash(self, eq=None, contains=None, ends_with=None) ->  'node_types.FQ':
        self._sha256_hash.extend(
            _str_cmps("sha256_hash", eq, contains, ends_with)
        )
        return self

    def with_creator(self, creator: 'node_types.PQ') ->  'node_types.FQ':
        creator = deepcopy(creator)
        self._creator = creator
        return self

    def with_deleter(self, deleter: 'node_types.PQ') ->  'node_types.FQ':
        deleter = deepcopy(deleter)
        self._deleter = deleter
        deleter._deleted_files = self
        return self

    def with_writers(self, writers: 'node_types.PQ') ->  'node_types.FQ':
        writers = deepcopy(writers)
        self._writers = writers
        return self

    def with_readers(self, readers: 'node_types.PQ') ->  'node_types.FQ':
        readers = deepcopy(readers)
        self._readers = readers
        readers._read_files = self
        return self

    def with_spawned_from(self, spawned_from: 'node_types.PQ') ->  'node_types.FQ':
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

    def query_first(self, dgraph_client, contains_node_key=None) -> Optional['node_types.FV']:
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

            creator: Optional[List['node_types.PV']] = None,
            deleter: Optional[List['node_types.PV']] = None,
            writers: Optional[List['node_types.PV']] = None,
            readers: Optional[List['node_types.PV']] = None,
            spawned_from: Optional[List['node_types.PV']] = None,
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
    def from_dict(dgraph_client: DgraphClient, d: Dict[str, Any]) -> 'node_types.FV':

        raw_creator = d.get("~created_file", None)
        raw_deleter = d.get("~deleted_file", None)
        raw_writers = d.get("~wrote_to_files", None)
        raw_readers = d.get("~read_files", None)
        raw_spawned_from = d.get("~bin_file", None)

        creator = None  # type: Optional[List['node_types.PV']]
        if raw_creator:
            creator = process_node.ProcessView.from_dict(dgraph_client, raw_creator)

        deleter = None  # type: Optional[List['node_types.PV']]
        if raw_deleter:
            deleter = process_node.ProcessView.from_dict(dgraph_client, raw_deleter)

        writers = None  # type: Optional[List['node_types.PV']]
        if raw_writers:
            writers = [
                process_node.ProcessView.from_dict(dgraph_client, raw) for raw in raw_writers
            ]

        readers = None  # type: Optional[List['node_types.PV']]
        if raw_readers:
            readers = [
                process_node.ProcessView.from_dict(dgraph_client, raw) for raw in raw_readers
            ]

        spawned_from = None  # type: Optional[List['node_types.PV']]
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


