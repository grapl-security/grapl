from typing import *

# noinspection Mypy
from pydgraph import DgraphClient

from grapl_analyzerlib.nodes.queryable import Queryable
from grapl_analyzerlib.nodes.viewable import Viewable

T = TypeVar("T")


class _ProcessQuery(Queryable[T]):
    def __init__(self) -> None:

        super(_ProcessQuery, self).__init__(_ProcessView)
        self._process_id = []  # type: List[List[Cmp[int]]]
        self._created_timestamp = []  # type: List[List[Cmp[int]]]
        self._asset_id = []  # type: List[List[Cmp[str]]]
        self._terminate_time = []  # type: List[List[Cmp[int]]]
        self._image_name = []  # type: List[List[Cmp[str]]]
        self._process_name = []  # type: List[List[Cmp[str]]]
        self._arguments = []  # type: List[List[Cmp[str]]]

        self._children = None  # type: Optional['_ProcessQuery[T]']
        self._bin_file = None  # type: Optional['_FileQuery[T]']
        self._created_files = None  # type: Optional['_FileQuery[T]']
        self._deleted_files = None  # type: Optional['_FileQuery[T]']
        self._read_files = None  # type: Optional['_FileQuery[T]']
        self._wrote_files = None  # type: Optional['_FileQuery[T]']
        self._created_connections = None  # type: Optional['_ExternalIpQuery[T]']
        self._bound_connection = None  # type: Optional['_ProcessQuery[T]']

        # Reverse edges
        self._parent = None  # type: Optional['_ProcessQuery[T]']

    def with_process_name(
            self,
            eq: Optional['StrCmp'] = None,
            contains: Optional['StrCmp'] = None,
            ends_with: Optional['StrCmp'] = None,
    ) -> '_ProcessQuery[T]':
        self._process_name.extend(_str_cmps("process_name", eq, contains, ends_with))
        return self

    def with_process_id(
            self,
            eq: Optional['IntCmp'] = None,
            gt: Optional['IntCmp'] = None,
            lt: Optional['IntCmp'] = None,
    ) -> '_ProcessQuery[T]':
        self._process_id.extend(_int_cmps("process_id", eq, gt, lt))
        return self

    def with_created_timestamp(
            self,
            eq: Optional['IntCmp'] = None,
            gt: Optional['IntCmp'] = None,
            lt: Optional['IntCmp'] = None,
    ) -> 'ProcessQuery':
        self._created_timestamp.extend(_int_cmps('created_timestamp', eq, gt, lt))
        return self

    def with_asset_id(
            self,
            eq: Optional['StrCmp'] = None,
            contains: Optional['StrCmp'] = None,
            ends_with: Optional['StrCmp'] = None,
    ) -> 'ProcessQuery':
        self._asset_id.extend(_str_cmps('asset_id', eq, contains, ends_with))
        return self

    def with_terminate_time(
            self,
            eq: Optional['IntCmp'] = None,
            gt: Optional['IntCmp'] = None,
            lt: Optional['IntCmp'] = None,
    ) -> 'ProcessQuery':
        self._terminate_time.extend(_int_cmps('terminate_time', eq, gt, lt))
        return self

    def with_image_name(
            self,
            eq: Optional['StrCmp'] = None,
            contains: Optional['StrCmp'] = None,
            ends_with: Optional['StrCmp'] = None,
    ) -> 'ProcessQuery':
        self._image_name.extend(_str_cmps('image_name', eq, contains, ends_with))
        return self

    def with_arguments(
            self,
            eq: Optional['StrCmp'] = None,
            contains: Optional['StrCmp'] = None,
            ends_with: Optional['StrCmp'] = None,
    ) -> 'ProcessQuery':
        self._arguments.extend(_str_cmps('arguments', eq, contains, ends_with))
        return self

    def with_children(self, child_query: Optional['_ProcessQuery[T]'] = None) -> '_ProcessQuery[T]':
        children = child_query or ProcessQuery()  # type: _ProcessQuery[T]
        children._parent = self
        self._children = children
        return self

    def with_bin_file(
            self,
            bin_file_query: Optional['_FileQuery[T]'] = None
    ) -> '_ProcessQuery[T]':
        bin_file = bin_file_query or FileQuery()  # type: _FileQuery[T]
        bin_file._spawned_from = self
        self._bin_file = bin_file
        return self

    def with_created_files(
            self,
            created_files_query: Optional['_FileQuery[T]'] = None
    ) -> '_ProcessQuery[T]':
        created_files = created_files_query or FileQuery()
        created_files._creator = self
        self._created_files = created_files
        return self

    def with_deleted_files(
            self,
            deleted_files_query: Optional['_FileQuery[T]'] = None
    ) -> '_ProcessQuery[T]':
        deleted_files = deleted_files_query or FileQuery()
        deleted_files._deleter = self
        self._deleted_files = deleted_files
        return self

    def with_read_files(
            self,
            read_files_query: Optional['_FileQuery[T]'] = None
    ) -> '_ProcessQuery[T]':

        read_files = read_files_query or FileQuery()

        read_files._readers = self
        self._read_files = read_files
        return self

    def with_wrote_files(
            self,
            wrote_files_query: Optional['_FileQuery[T]'] = None
    ) -> '_ProcessQuery[T]':
        wrote_files = wrote_files_query or FileQuery()

        wrote_files._writers = self
        self._wrote_files = wrote_files
        return self

    def with_created_connection(
            self,
            created_connection_query: Optional['_ExternalIpQuery[T]']  = None
    ) -> '_ProcessQuery[T]':
        created_connections = created_connection_query or  _ExternalIpQuery()  # type: _ExternalIpQuery[T]
        created_connections._connections_from = self
        self._created_connections = created_connections
        return self

    # def with_bound_connection(
    #         self,
    #         bound_connection_query: Optional['_ProcessQuery[T]'] = None
    # ) -> '_ProcessQuery[T]':
    #     if bound_connection_query is None:
    #         bound_connection = ProcessQuery()  # type: _ProcessQuery[T]
    #     else:
    #         bound_connection = deepcopy(bound_connection_query)
    #     children._parent = self
    #     self._children = bound_connection
    #     return self

    def with_parent(self, parent_query: Optional['_ProcessQuery[T]'] = None) -> '_ProcessQuery[T]':
        parent = parent_query or ProcessQuery()  # type: _ProcessQuery[T]

        parent._children = self
        self._parent = parent
        return self

    def _get_unique_predicate(self) -> Optional[Tuple[str, 'PropertyT']]:
        return "process_id", int

    def _get_node_type_name(self) -> Optional[str]:
        return None

    def _get_property_filters(self) -> Mapping[str, 'PropertyFilter[Property]']:
        props = {
            'process_id': self._process_id,
            'process_name': self._process_name,
            'created_timestamp': self._created_timestamp,
            'asset_id': self._asset_id,
            'terminate_time': self._terminate_time,
            'image_name': self._image_name,
            'arguments': self._arguments,
        }
        combined = {}
        for prop_name, prop_filter in props.items():
            if prop_filter:
                combined[prop_name] = cast('PropertyFilter[Property]', prop_filter)

        return combined

    def _get_forward_edges(self) -> Mapping[str, "Queryable[T]"]:
        forward_edges = {
            "children": self._children,
            "bin_file": self._bin_file,
            "created_files": self._created_files,
            "deleted_files": self._deleted_files,
            "read_files": self._read_files,
            "wrote_files": self._wrote_files,
            "created_connections": self._created_connections,
            "bound_connection": self._bound_connection,
        }

        return {fe[0]: fe[1] for fe in forward_edges.items() if fe[1] is not None}

    def _get_reverse_edges(self) -> Mapping[str, Tuple["Queryable[T]", str]]:
        reverse_edges = {
            "~children": (self._parent, "parent"),
        }

        return {fe[0]: (fe[1][0], fe[1][1]) for fe in reverse_edges.items() if fe[1][0] is not None}

    def query(
            self,
            dgraph_client: DgraphClient,
            contains_node_key: Optional[str] = None,
            first: Optional[int] = 1000,
    ) -> List['ProcessView']:
        return self._query(
            dgraph_client,
            contains_node_key,
            first
        )

    def query_first(
            self, dgraph_client: DgraphClient, contains_node_key: Optional[str] = None
    ) -> Optional['ProcessView']:
        return self._query_first(dgraph_client, contains_node_key)


class _ProcessView(Viewable[T]):

    def __init__(
            self,
            dgraph_client: DgraphClient,
            uid: str,
            node_key: str,
            node_type: Optional[str] = None,
            process_id: Optional[int] = None,
            created_timestamp: Optional[int] = None,
            asset_id: Optional[str] = None,
            terminate_time: Optional[int] = None,
            image_name: Optional[str] = None,
            process_name: Optional[str] = None,
            arguments: Optional[str] = None,
            children: Optional[List['_ProcessView[T]']] = None,
            bin_file: Optional['_FileView[T]'] = None,
            created_files: Optional[List['_FileView[T]']] = None,
            read_files: Optional[List['_FileView[T]']] = None,
            wrote_to_files: Optional[List['_FileView[T]']] = None,
            deleted_files: Optional[List['_FileView[T]']] = None,
            created_connections: Optional[List['_ExternalIpView[T]']] = None,
            parent: Optional['_ProcessView[T]'] = None,
    ) -> None:
        super(_ProcessView, self).__init__(dgraph_client, node_key=node_key, uid=uid)
        self.process_id = process_id
        self.node_type = node_type
        self.created_timestamp = created_timestamp
        self.asset_id = asset_id
        self.terminate_time = terminate_time
        self.image_name = image_name
        self.process_name = process_name
        self.arguments = arguments

        self.children = children or []
        self.created_files = created_files or []
        self.read_files = read_files or []
        self.wrote_to_files = wrote_to_files or []
        self.deleted_files = deleted_files or []
        self.created_connections = created_connections or []
        self.bin_file = bin_file
        self.parent = parent

    def get_process_id(self) -> Optional[int]:
        if self.process_id is not None:
            return self.process_id
        self.process_id = cast(int, self.fetch_property('process_id', int))
        return self.process_id

    def get_process_name(self) -> Optional[str]:
        if self.process_name is not None:
            return self.process_name
        self.process_name = cast(str, self.fetch_property('process_name', str))
        return self.process_name

    def get_created_timestamp(self) -> Optional[int]:
        if self.created_timestamp is not None:
            return self.created_timestamp
        self.created_timestamp = cast(int, self.fetch_property('created_timestamp', int))
        return self.created_timestamp

    def get_asset_id(self) -> Optional[str]:
        if self.asset_id is not None:
            return self.asset_id
        self.asset_id = cast(str, self.fetch_property('asset_id', str))
        return self.asset_id

    def get_terminate_time(self) -> Optional[int]:
        if self.terminate_time is not None:
            return self.terminate_time
        self.terminate_time = cast(int, self.fetch_property('terminate_time', int))
        return self.terminate_time

    def get_image_name(self) -> Optional[str]:
        if self.image_name is not None:
            return self.image_name
        self.image_name = cast(str, self.fetch_property('image_name', str))
        return self.image_name

    def get_arguments(self) -> Optional[str]:
        if self.arguments is not None:
            return self.arguments
        self.arguments = cast(str, self.fetch_property('arguments', str))
        return self.arguments

    def get_children(self) -> 'List[ProcessView]':
        self.children = cast('List[ProcessView]', self.fetch_edges('children', ProcessView))
        return self.children

    def get_created_connections(self) -> 'List[ExternalIpView]':
        self.created_connections = cast(
            'List[ExternalIpView]',
            self.fetch_edges('created_connections', ExternalIpView)
        )
        return self.created_connections

    def get_bin_file(self) -> 'Optional[FileView]':
        self.bin_file = cast('Optional[FileView]', self.fetch_edge('bin_file', FileView))
        return self.bin_file

    def get_created_files(self) -> 'List[FileView]':
        self.created_files = cast('List[FileView]', self.fetch_edges('created_files', FileView))
        return self.created_files

    def get_read_files(self) -> 'List[FileView]':
        self.read_files = cast('List[FileView]', self.fetch_edges('read_files', FileView))
        return self.read_files

    def get_wrote_to_files(self) -> 'List[FileView]':
        self.wrote_to_files = cast('List[FileView]', self.fetch_edges('wrote_to_files', FileView))
        return self.wrote_to_files

    def get_deleted_files(self) -> 'List[FileView]':
        self.deleted_files = cast('List[FileView]', self.fetch_edges('deleted_files', FileView))
        return self.deleted_files

    def get_parent(self) -> Optional['ProcessView']:
        self.parent = cast('Optional[ProcessView]', self.fetch_edge('~children', ProcessView))
        return self.parent

    @staticmethod
    def _get_property_types() -> Mapping[str, "PropertyT"]:
        return {
            'process_id': int,
            'process_name': str,
            'created_timestamp': int,
            'asset_id': str,
            'terminate_time': int,
            'image_name': str,
            'arguments': str,
        }

    @staticmethod
    def _get_forward_edge_types() -> Mapping[str, "_EdgeViewT[T]"]:
        return {
            'children': [_ProcessView],
            'bin_file': _FileView,
            'created_files': [_FileView],
            'created_connections': [_ExternalIpView],
            'read_files': [_FileView],
            'wrote_to_files': [_FileView],
            'deleted_files': [_FileView],
        }

    @staticmethod
    def _get_reverse_edge_types() -> Mapping[str, Tuple["_EdgeViewT[T]", str]]:
        return {
            '~children': (_ProcessView, 'parent')
        }

    def _get_properties(self, fetch: bool = False) -> Mapping[str, Union[str, int]]:
        # TODO: Fetch it `fetch`
        _props = {
            'process_id': self.process_id,
            'process_name': self.process_name,
            'created_timestamp': self.created_timestamp,
            'asset_id': self.asset_id,
            'terminate_time': self.terminate_time,
            'image_name': self.image_name,
            'arguments': self.arguments,
        }

        props = {p[0]: p[1] for p in _props.items() if p[1] is not None}  # type: Mapping[str, Union[str, int]]

        return props

    def _get_forward_edges(self) -> 'Mapping[str, _ForwardEdgeView[T]]':
        f_edges = {
            'children': self.children,
            'created_files': self.created_files,
            'created_connections': self.created_connections,
            'read_files': self.read_files,
            'wrote_to_files': self.wrote_to_files,
            'deleted_files': self.deleted_files,
            'bin_file': self.bin_file,
        }

        forward_edges = {name: value for name, value in f_edges.items() if value is not None}
        return cast('Mapping[str, _ForwardEdgeView[T]]', forward_edges)

    def _get_reverse_edges(self) -> 'Mapping[str, _ReverseEdgeView[T]]':
        _reverse_edges = {
            '~children': (self.parent, 'parent')
        }

        reverse_edges = {name: value for name, value in _reverse_edges.items() if value[0] is not None}
        return cast('Mapping[str, _ReverseEdgeView[T]]', reverse_edges)



ProcessQuery = _ProcessQuery[Any]
ProcessView = _ProcessView[Any]

from grapl_analyzerlib.nodes.file_node import _FileView, _FileQuery, FileQuery, FileView
from grapl_analyzerlib.nodes.comparators import PropertyFilter, Cmp, StrCmp, _str_cmps, IntCmp, _int_cmps
from grapl_analyzerlib.nodes.types import PropertyT, Property
from grapl_analyzerlib.nodes.viewable import _EdgeViewT, _ForwardEdgeView, _ReverseEdgeView
from grapl_analyzerlib.nodes.external_ip_node import _ExternalIpQuery, ExternalIpView, _ExternalIpView
