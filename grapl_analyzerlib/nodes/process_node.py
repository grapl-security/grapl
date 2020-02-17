from typing import *

# noinspection Mypy
from pydgraph import DgraphClient

from grapl_analyzerlib.nodes.queryable import Queryable, NQ
from grapl_analyzerlib.nodes.viewable import Viewable, NV

T = TypeVar("T")

IProcessQuery = TypeVar("IProcessQuery", bound="ProcessQuery")
IProcessView = TypeVar("IProcessView", bound="ProcessView")


class ProcessQuery(Queryable[IProcessView]):
    def __init__(self) -> None:
        super(ProcessQuery, self).__init__(ProcessView)
        self._process_id = []  # type: List[List[Cmp[int]]]
        self._created_timestamp = []  # type: List[List[Cmp[int]]]
        self._asset_id = []  # type: List[List[Cmp[str]]]
        self._terminate_time = []  # type: List[List[Cmp[int]]]
        self._image_name = []  # type: List[List[Cmp[str]]]
        self._process_name = []  # type: List[List[Cmp[str]]]
        self._arguments = []  # type: List[List[Cmp[str]]]

        self._children = None  # type: Optional['ProcessQuery']
        self._bin_file = None  # type: Optional['FileQuery']
        self._created_files = None  # type: Optional['FileQuery']
        self._deleted_files = None  # type: Optional['FileQuery']
        self._read_files = None  # type: Optional['FileQuery']
        self._wrote_files = None  # type: Optional['FileQuery']
        self._created_connections = (
            None
        )  # type: Optional['IProcessOutboundConnectionQuery']
        self._inbound_connections = (
            None
        )  # type: Optional['IProcessInboundConnectionQuery']

        # Reverse edges
        self._parent = None  # type: Optional['ProcessQuery']

    def with_process_name(
        self: "NQ",
        eq: Optional["StrCmp"] = None,
        contains: Optional["StrCmp"] = None,
        ends_with: Optional["StrCmp"] = None,
        starts_with: Optional["StrCmp"] = None,
        regexp: Optional["StrCmp"] = None,
        distance: Optional[Tuple["StrCmp", int]] = None,
    ) -> "NQ":
        cast(ProcessQuery, self)._process_name.extend(
            _str_cmps(
                "process_name",
                eq=eq,
                contains=contains,
                ends_with=ends_with,
                starts_with=starts_with,
                regexp=regexp,
                distance=distance,
            )
        )
        return self

    def with_process_id(
        self: "NQ",
        eq: Optional["IntCmp"] = None,
        gt: Optional["IntCmp"] = None,
        lt: Optional["IntCmp"] = None,
    ) -> "NQ":
        cast(ProcessQuery, self)._process_id.extend(_int_cmps("process_id", eq, gt, lt))
        return self

    def with_created_timestamp(
        self: "NQ",
        eq: Optional["IntCmp"] = None,
        gt: Optional["IntCmp"] = None,
        lt: Optional["IntCmp"] = None,
    ) -> "NQ":
        cast(ProcessQuery, self)._created_timestamp.extend(
            _int_cmps("created_timestamp", eq, gt, lt)
        )
        return self

    def with_asset_id(
        self: "NQ",
        eq: Optional["StrCmp"] = None,
        contains: Optional["StrCmp"] = None,
        ends_with: Optional["StrCmp"] = None,
        starts_with: Optional["StrCmp"] = None,
        regexp: Optional["StrCmp"] = None,
        distance: Optional[Tuple["StrCmp", int]] = None,
    ) -> "NQ":
        cast(ProcessQuery, self)._asset_id.extend(
            _str_cmps(
                "asset_id",
                eq=eq,
                contains=contains,
                ends_with=ends_with,
                starts_with=starts_with,
                regexp=regexp,
                distance=distance,
            )
        )
        return self

    def with_terminate_time(
        self: "NQ",
        eq: Optional["IntCmp"] = None,
        gt: Optional["IntCmp"] = None,
        lt: Optional["IntCmp"] = None,
    ) -> "NQ":
        cast(ProcessQuery, self)._terminate_time.extend(
            _int_cmps("terminate_time", eq, gt, lt)
        )
        return self

    def with_image_name(
        self: "NQ",
        eq: Optional["StrCmp"] = None,
        contains: Optional["StrCmp"] = None,
        ends_with: Optional["StrCmp"] = None,
        starts_with: Optional["StrCmp"] = None,
        regexp: Optional["StrCmp"] = None,
        distance: Optional[Tuple["StrCmp", int]] = None,
    ) -> "NQ":
        cast(ProcessQuery, self)._image_name.extend(
            _str_cmps(
                "image_name",
                eq=eq,
                contains=contains,
                ends_with=ends_with,
                starts_with=starts_with,
                regexp=regexp,
                distance=distance,
            )
        )
        return self

    def with_arguments(
        self: "NQ",
        eq: Optional["StrCmp"] = None,
        contains: Optional["StrCmp"] = None,
        ends_with: Optional["StrCmp"] = None,
        starts_with: Optional["StrCmp"] = None,
        regexp: Optional["StrCmp"] = None,
        distance: Optional[Tuple["StrCmp", int]] = None,
    ) -> "NQ":
        cast(ProcessQuery, self)._arguments.extend(
            _str_cmps(
                "arguments",
                eq=eq,
                contains=contains,
                ends_with=ends_with,
                starts_with=starts_with,
                regexp=regexp,
                distance=distance,
            )
        )
        return self

    def with_children(
        self: "NQ", child_query: Optional["IProcessQuery"] = None
    ) -> "NQ":
        children = child_query or ProcessQuery()  # type: ProcessQuery
        children._parent = cast(ProcessQuery, self)
        cast(ProcessQuery, self)._children = children
        return self

    def with_bin_file(
        self: "NQ", bin_file_query: Optional["IFileQuery"] = None
    ) -> "NQ":
        bin_file = bin_file_query or FileQuery()  # type: FileQuery
        bin_file._spawned_from = cast(ProcessQuery, self)
        cast(ProcessQuery, self)._bin_file = bin_file
        return self

    def with_created_files(
        self: "NQ", created_files_query: Optional["IFileQuery"] = None
    ) -> "NQ":
        created_files = created_files_query or FileQuery()
        created_files._creator = cast(ProcessQuery, self)
        cast(ProcessQuery, self)._created_files = created_files
        return self

    def with_deleted_files(
        self: "NQ", deleted_files_query: Optional["IFileQuery"] = None
    ) -> "NQ":
        deleted_files = deleted_files_query or FileQuery()
        deleted_files._deleter = cast(ProcessQuery, self)
        cast(ProcessQuery, self)._deleted_files = deleted_files
        return self

    def with_read_files(
        self: "NQ", read_files_query: Optional["IFileQuery"] = None
    ) -> "NQ":

        read_files = read_files_query or FileQuery()

        read_files._readers = cast(ProcessQuery, self)
        cast(ProcessQuery, self)._read_files = read_files
        return self

    def with_wrote_files(
        self: "NQ", wrote_files_query: Optional["IFileQuery"] = None
    ) -> "NQ":
        wrote_files = wrote_files_query or FileQuery()

        wrote_files._writers = cast(ProcessQuery, self)
        cast(ProcessQuery, self)._wrote_files = wrote_files
        return self

    def with_created_connections(
        self: "NQ",
        created_connection_query: Optional[
            "IProcessOutboundConnectionQuery"
        ] = None,
    ) -> "NQ":
        from grapl_analyzerlib.nodes.process_outbound_network_connection import (
            ProcessOutboundConnectionQuery,
        )

        created_connections = (
            created_connection_query or ProcessOutboundConnectionQuery()
        )  # type: ProcessOutboundConnectionQuery
        created_connections._connecting_processes = self
        self._created_connections = created_connections
        return self

    def with_inbound_connections(
        self: "NQ",
        inbound_connection_query: Optional[
            "IProcessInboundConnectionQuery"
        ] = None,
    ) -> "NQ":
        from grapl_analyzerlib.nodes.process_inbound_network_connection import (
            ProcessInboundConnectionQuery,
        )

        inbound_connection = (
            inbound_connection_query or ProcessInboundConnectionQuery()
        )  # type: ProcessInboundConnectionQuery
        inbound_connection._bound_by = self
        self._inbound_connections = inbound_connection
        return self

    def with_parent(self: "NQ", parent_query: Optional["IProcessQuery"] = None) -> "NQ":
        parent = parent_query or ProcessQuery()  # type: ProcessQuery

        parent._children = cast(ProcessQuery, self)
        cast(ProcessQuery, self)._parent = parent
        return self

    def _get_unique_predicate(self) -> Optional[Tuple[str, "PropertyT"]]:
        return "process_id", int

    def _get_node_type_name(self) -> str:
        return "Process"

    def _get_property_filters(self) -> Mapping[str, "PropertyFilter[Property]"]:
        props = {
            "node_key": self._node_key,
            "process_id": self._process_id,
            "process_name": self._process_name,
            "created_timestamp": self._created_timestamp,
            "asset_id": self._asset_id,
            "terminate_time": self._terminate_time,
            "image_name": self._image_name,
            "arguments": self._arguments,
        }
        combined = {}
        for prop_name, prop_filter in props.items():
            if prop_filter:
                combined[prop_name] = cast("PropertyFilter[Property]", prop_filter)

        return combined

    def _get_forward_edges(self) -> Mapping[str, "Queryable"]:
        forward_edges = {
            "children": self._children,
            "bin_file": self._bin_file,
            "created_files": self._created_files,
            "deleted_files": self._deleted_files,
            "read_files": self._read_files,
            "wrote_files": self._wrote_files,
            "created_connections": self._created_connections,
            "inbound_connections": self._inbound_connections,
        }

        return {fe[0]: fe[1] for fe in forward_edges.items() if fe[1] is not None}

    def _get_reverse_edges(self) -> Mapping[str, Tuple["Queryable", str]]:
        reverse_edges = {"~children": (self._parent, "parent")}

        return {
            fe[0]: (fe[1][0], fe[1][1])
            for fe in reverse_edges.items()
            if fe[1][0] is not None
        }


class ProcessView(Viewable):
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
        children: Optional[List["NV"]] = None,
        bin_file: Optional["FileView"] = None,
        created_files: Optional[List["FileView"]] = None,
        read_files: Optional[List["FileView"]] = None,
        wrote_files: Optional[List["FileView"]] = None,
        deleted_files: Optional[List["FileView"]] = None,
        created_connections: Optional[
            List["ProcessOutboundConnectionQuery"]
        ] = None,
        inbound_connections: Optional[
            List["ProcessInboundConnectionQuery"]
        ] = None,
        parent: Optional["NV"] = None,
    ) -> None:
        super(ProcessView, self).__init__(dgraph_client, node_key=node_key, uid=uid)
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
        self.wrote_files = wrote_files or []
        self.deleted_files = deleted_files or []
        self.created_connections = created_connections or []
        self.inbound_connections = inbound_connections or []
        self.bin_file = bin_file
        self.parent = parent

    def get_node_type(self) -> str:
        return 'Process'

    def get_process_id(self: "NV") -> Optional[int]:
        if cast(ProcessView, self).process_id is not None:
            return cast(ProcessView, self).process_id
        cast(ProcessView, self).process_id = cast(
            int, self.fetch_property("process_id", int)
        )
        return cast(ProcessView, self).process_id

    def get_process_name(self: "NV") -> Optional[str]:
        if cast(ProcessView, self).process_name is not None:
            return cast(ProcessView, self).process_name
        cast(ProcessView, self).process_name = cast(
            str, self.fetch_property("process_name", str)
        )
        return cast(ProcessView, self).process_name

    def get_created_timestamp(self: "NV") -> Optional[int]:
        if cast(ProcessView, self).created_timestamp is not None:
            return cast(ProcessView, self).created_timestamp
        cast(ProcessView, self).created_timestamp = cast(
            int, self.fetch_property("created_timestamp", int)
        )
        return cast(ProcessView, self).created_timestamp

    def get_asset_id(self: "NV") -> Optional[str]:
        if cast(ProcessView, self).asset_id is not None:
            return cast(ProcessView, self).asset_id
        cast(ProcessView, self).asset_id = cast(
            str, self.fetch_property("asset_id", str)
        )
        return cast(ProcessView, self).asset_id

    def get_terminate_time(self: "NV") -> Optional[int]:
        if cast(ProcessView, self).terminate_time is not None:
            return cast(ProcessView, self).terminate_time
        cast(ProcessView, self).terminate_time = cast(
            int, self.fetch_property("terminate_time", int)
        )
        return cast(ProcessView, self).terminate_time

    def get_image_name(self: "NV") -> Optional[str]:
        if cast(ProcessView, self).image_name is not None:
            return cast(ProcessView, self).image_name
        cast(ProcessView, self).image_name = cast(
            str, self.fetch_property("image_name", str)
        )
        return cast(ProcessView, self).image_name

    def get_arguments(self: "NV") -> Optional[str]:
        if cast(ProcessView, self).arguments is not None:
            return cast(ProcessView, self).arguments
        cast(ProcessView, self).arguments = cast(
            str, self.fetch_property("arguments", str)
        )
        return cast(ProcessView, self).arguments

    def get_children(
        self: "NV", match_children: Optional["IProcessQuery"] = None
    ) -> "List[NV]":
        query = ProcessQuery()
        query.view_type = type(self)
        _match_children = match_children or ProcessQuery()  # type: ProcessQuery
        _match_children.view_type = type(self)

        self_node = (
            ProcessQuery()
            .with_node_key(eq=self.node_key)
            .with_children(_match_children or ProcessQuery())
            .query_first(self.dgraph_client)
        )

        if self_node:
            cast(ProcessView, self).children = self_node.children

        return cast(ProcessView, self).children

    def get_created_connections(
        self: "NV"
    ) -> "List[ProcessOutboundConnectionView]":
        from grapl_analyzerlib.nodes.process_outbound_network_connection import (
            ProcessOutboundConnectionView,
        )

        cast(ProcessView, self).created_connections = cast(
            List[ProcessOutboundConnectionView],
            self.fetch_edges(
                "created_connections", ProcessOutboundConnectionView
            ),
        )
        return cast(ProcessView, self).created_connections

    def get_inbound_connections(
        self: "NV"
    ) -> "List[ProcessInboundConnectionView]":
        cast(ProcessView, self).created_connections = cast(
            List[ProcessInboundConnectionView],
            self.fetch_edges(
                "inbound_connections", ProcessInboundConnectionView
            ),
        )
        return cast(ProcessView, self).created_connections

    def get_bin_file(self: "NV") -> "Optional[FileView]":
        cast(ProcessView, self).bin_file = cast(
            Optional[FileView], self.fetch_edge("bin_file", FileView)
        )
        return cast(ProcessView, self).bin_file

    def get_created_files(self: "NV") -> "List[FileView]":
        cast(ProcessView, self).created_files = cast(
            List[FileView], self.fetch_edges("created_files", FileView)
        )
        return cast(ProcessView, self).created_files

    def get_read_files(self: "NV") -> "List[FileView]":
        cast(ProcessView, self).read_files = cast(
            List[FileView], self.fetch_edges("read_files", FileView)
        )
        return cast(ProcessView, self).read_files

    def get_wrote_files(self: "NV") -> "List[FileView]":
        cast(ProcessView, self).wrote_files = cast(
            List[FileView], self.fetch_edges("wrote_files", FileView)
        )
        return cast(ProcessView, self).wrote_files

    def get_deleted_files(self: "NV") -> "List[FileView]":
        cast(ProcessView, self).deleted_files = cast(
            List[FileView], self.fetch_edges("deleted_files", FileView)
        )
        return cast(ProcessView, self).deleted_files

    def get_parent(
        self: "NV", match_parent: Optional["IProcessQuery"] = None
    ) -> Optional["NV"]:
        query = ProcessQuery()
        query.view_type = type(self)
        _match_parent = match_parent or ProcessQuery()  # type: ProcessQuery
        _match_parent.view_type = type(self)

        self_node = (
            ProcessQuery()
            .with_node_key(eq=self.node_key)
            .with_parent(_match_parent)
            .query_first(self.dgraph_client)
        )

        if self_node:
            cast(ProcessView, self).parent = self_node.parent

            assert self.node_key == self_node.node_key, 'self and self_node do not have matching node_keys'

        return cast(ProcessView, self).parent

    @staticmethod
    def _get_property_types() -> Mapping[str, "PropertyT"]:
        return {
            "process_id": int,
            "process_name": str,
            "created_timestamp": int,
            "asset_id": str,
            "terminate_time": int,
            "image_name": str,
            "arguments": str,
        }

    @staticmethod
    def _get_forward_edge_types() -> Mapping[str, "EdgeViewT"]:
        from grapl_analyzerlib.nodes.process_inbound_network_connection import (
            ProcessInboundConnectionView,
        )
        from grapl_analyzerlib.nodes.process_outbound_network_connection import (
            ProcessOutboundConnectionView,
        )

        return {
            "children": [ProcessView],
            "bin_file": FileView,
            "created_files": [FileView],
            "created_connections": [ProcessOutboundConnectionView],
            "inbound_connections": [ProcessInboundConnectionView],
            "read_files": [FileView],
            "wrote_files": [FileView],
            "deleted_files": [FileView],
        }

    @staticmethod
    def _get_reverse_edge_types() -> Mapping[str, Tuple["EdgeViewT", str]]:
        return {"~children": (ProcessView, "parent")}

    def _get_properties(self, fetch: bool = False) -> Mapping[str, Union[str, int]]:
        # TODO: Fetch it `fetch`
        _props = {
            "process_id": self.process_id,
            "process_name": self.process_name,
            "created_timestamp": self.created_timestamp,
            "asset_id": self.asset_id,
            "terminate_time": self.terminate_time,
            "image_name": self.image_name,
            "arguments": self.arguments,
        }

        props = {
            p[0]: p[1] for p in _props.items() if p[1] is not None
        }  # type: Mapping[str, Union[str, int]]

        return props

    def _get_forward_edges(self) -> "Mapping[str, ForwardEdgeView]":
        f_edges = {
            "children": self.children,
            "created_files": self.created_files,
            "created_connections": self.created_connections,
            "inbound_connections": self.inbound_connections,
            "read_files": self.read_files,
            "wrote_files": self.wrote_files,
            "deleted_files": self.deleted_files,
            "bin_file": self.bin_file,
        }

        forward_edges = {
            name: value for name, value in f_edges.items() if value is not None
        }
        return cast("Mapping[str, ForwardEdgeView]", forward_edges)

    def _get_reverse_edges(self) -> "Mapping[str, ReverseEdgeView]":
        _reverse_edges = {"~children": (self.parent, "parent")}

        reverse_edges = {
            name: value
            for name, value in _reverse_edges.items()
            if value[0] is not None
        }
        return cast("Mapping[str, ReverseEdgeView]", reverse_edges)


from grapl_analyzerlib.nodes.file_node import FileQuery, FileView, IFileQuery
from grapl_analyzerlib.nodes.comparators import (
    PropertyFilter,
    Cmp,
    StrCmp,
    _str_cmps,
    IntCmp,
    _int_cmps,
)
from grapl_analyzerlib.nodes.types import PropertyT, Property
from grapl_analyzerlib.nodes.viewable import EdgeViewT, ForwardEdgeView, ReverseEdgeView

from grapl_analyzerlib.nodes.process_inbound_network_connection import (
    IProcessInboundConnectionQuery,
    ProcessInboundConnectionView,
    ProcessInboundConnectionQuery)
from grapl_analyzerlib.nodes.process_outbound_network_connection import (
    ProcessOutboundConnectionView,
    IProcessOutboundConnectionQuery,
    ProcessOutboundConnectionQuery)
