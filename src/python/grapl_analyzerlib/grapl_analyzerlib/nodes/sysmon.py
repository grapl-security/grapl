from __future__ import annotations
from typing import Optional, Any, Set, List, Dict, Tuple
import grapl_analyzerlib
import grapl_analyzerlib.node_types
import grapl_analyzerlib.nodes.entity
import grapl_analyzerlib.queryable


def default_file_properties() -> Dict[str, grapl_analyzerlib.node_types.PropType]:
    return {
        "machine_id": grapl_analyzerlib.node_types.PropType(
            grapl_analyzerlib.node_types.PropPrimitive.Str, False
        ),
        "path": grapl_analyzerlib.node_types.PropType(
            grapl_analyzerlib.node_types.PropPrimitive.Str, False
        ),
    }


def default_file_edges() -> Dict[str, Tuple[grapl_analyzerlib.node_types.EdgeT, str]]:
    return {
        "process_executed_from_image": (
            grapl_analyzerlib.node_types.EdgeT(
                FileSchema,
                ProcessSchema,
                grapl_analyzerlib.node_types.EdgeRelationship.ManyToMany,
            ),
            "process_image",
        ),
        "created_by_process": (
            grapl_analyzerlib.node_types.EdgeT(
                FileSchema,
                ProcessSchema,
                grapl_analyzerlib.node_types.EdgeRelationship.ManyToMany,
            ),
            "created_files",
        ),
        "file_on_machine": (
            grapl_analyzerlib.node_types.EdgeT(
                FileSchema,
                MachineSchema,
                grapl_analyzerlib.node_types.EdgeRelationship.ManyToMany,
            ),
            "machine_files",
        ),
    }


class FileSchema(grapl_analyzerlib.nodes.entity.EntitySchema):
    def __init__(self):
        super(FileSchema, self).__init__(
            default_file_properties(), default_file_edges(), lambda: FileView
        )

    @staticmethod
    def self_type() -> str:
        return "File"


class FileQuery(grapl_analyzerlib.nodes.entity.EntityQuery["FileView", "FileQuery"]):
    def with_machine_id(
        self,
        *,
        eq: Optional[grapl_analyzerlib.comparators.StrOrNot] = None,
        contains: Optional[
            "grapl_analyzerlib.comparators.OneOrMany[grapl_analyzerlib.comparators.StrOrNot]"
        ] = None,
        starts_with: Optional["grapl_analyzerlib.comparators.StrOrNot"] = None,
        ends_with: Optional["grapl_analyzerlib.comparators.StrOrNot"] = None,
        regexp: Optional[
            "grapl_analyzerlib.comparators.OneOrMany[grapl_analyzerlib.comparators.StrOrNot]"
        ] = None,
        distance_lt: Optional[Tuple[str, int]] = None,
    ):
        (
            self.with_str_property(
                "machine_id",
                eq=eq,
                contains=contains,
                starts_with=starts_with,
                ends_with=ends_with,
                regexp=regexp,
                distance_lt=distance_lt,
            )
        )
        return self

    def with_path(
        self,
        *,
        eq: Optional[grapl_analyzerlib.comparators.StrOrNot] = None,
        contains: Optional[
            "grapl_analyzerlib.comparators.OneOrMany[grapl_analyzerlib.comparators.StrOrNot]"
        ] = None,
        starts_with: Optional["grapl_analyzerlib.comparators.StrOrNot"] = None,
        ends_with: Optional["grapl_analyzerlib.comparators.StrOrNot"] = None,
        regexp: Optional[
            "grapl_analyzerlib.comparators.OneOrMany[grapl_analyzerlib.comparators.StrOrNot]"
        ] = None,
        distance_lt: Optional[Tuple[str, int]] = None,
    ):
        (
            self.with_str_property(
                "path",
                eq=eq,
                contains=contains,
                starts_with=starts_with,
                ends_with=ends_with,
                regexp=regexp,
                distance_lt=distance_lt,
            )
        )
        return self

    def with_process_executed_from_image(
        self: FileQuery, *process_executed_from_image: ProcessQuery
    ) -> FileQuery:
        return self.with_to_neighbor(
            FileQuery,
            "process_executed_from_image",
            "process_image",
            process_executed_from_image,
        )

    def with_created_by_process(
        self: FileQuery, *created_by_process: ProcessQuery
    ) -> FileQuery:
        return self.with_to_neighbor(
            FileQuery, "created_by_process", "created_files", created_by_process
        )

    def with_file_on_machine(
        self: FileQuery, *file_on_machine: MachineQuery
    ) -> FileQuery:
        return self.with_to_neighbor(
            FileQuery, "file_on_machine", "machine_files", file_on_machine
        )

    @classmethod
    def node_schema(cls) -> "grapl_analyzerlib.schema.Schema":
        return FileSchema()


class FileView(grapl_analyzerlib.nodes.entity.EntityView["FileView", "FileQuery"]):
    queryable = FileQuery

    def __init__(
        self,
        uid: int,
        node_key: str,
        graph_client: Any,
        node_types: Set[str],
        machine_id: Optional["str"] = None,
        path: Optional["str"] = None,
        process_executed_from_image: Optional[List["ProcessView"]] = None,
        created_by_process: Optional[List["ProcessView"]] = None,
        file_on_machine: Optional[List["MachineView"]] = None,
        **kwargs,
    ) -> None:
        super().__init__(uid, node_key, graph_client, node_types, **kwargs)
        if machine_id:
            self.set_predicate("machine_id", machine_id)
        if path:
            self.set_predicate("path", path)
        if process_executed_from_image:
            self.set_predicate(
                "process_executed_from_image", process_executed_from_image or []
            )
        if created_by_process:
            self.set_predicate("created_by_process", created_by_process or [])
        if file_on_machine:
            self.set_predicate("file_on_machine", file_on_machine or [])

    def get_machine_id(self, cached: bool = True) -> Optional[str]:
        return self.get_str("machine_id", cached=cached)

    def get_path(self, cached: bool = True) -> Optional[str]:
        return self.get_str("path", cached=cached)

    def get_process_executed_from_image(
        self, *process_executed_from_image: ProcessQuery, cached=False
    ) -> "List[ProcessView]":
        return self.get_neighbor(
            ProcessQuery,
            "process_executed_from_image",
            "process_image",
            process_executed_from_image,
            cached,
        )

    def get_created_by_process(
        self, *created_by_process: ProcessQuery, cached=False
    ) -> "List[ProcessView]":
        return self.get_neighbor(
            ProcessQuery,
            "created_by_process",
            "created_files",
            created_by_process,
            cached,
        )

    def get_file_on_machine(
        self, *file_on_machine: MachineQuery, cached=False
    ) -> "List[MachineView]":
        return self.get_neighbor(
            MachineQuery, "file_on_machine", "machine_files", file_on_machine, cached
        )

    @classmethod
    def node_schema(cls) -> "grapl_analyzerlib.schema.Schema":
        return FileSchema()


def default_machine_properties() -> Dict[str, grapl_analyzerlib.node_types.PropType]:
    return {
        "machine_id": grapl_analyzerlib.node_types.PropType(
            grapl_analyzerlib.node_types.PropPrimitive.Str, False
        ),
        "hostname": grapl_analyzerlib.node_types.PropType(
            grapl_analyzerlib.node_types.PropPrimitive.Str, False
        ),
        "os": grapl_analyzerlib.node_types.PropType(
            grapl_analyzerlib.node_types.PropPrimitive.Str, False
        ),
    }


def default_machine_edges() -> Dict[
    str, Tuple[grapl_analyzerlib.node_types.EdgeT, str]
]:
    return {
        "machine_process": (
            grapl_analyzerlib.node_types.EdgeT(
                MachineSchema,
                ProcessSchema,
                grapl_analyzerlib.node_types.EdgeRelationship.ManyToMany,
            ),
            "process_executed_by_machine",
        ),
        "machine_files": (
            grapl_analyzerlib.node_types.EdgeT(
                MachineSchema,
                FileSchema,
                grapl_analyzerlib.node_types.EdgeRelationship.ManyToMany,
            ),
            "file_on_machine",
        ),
    }


class MachineSchema(grapl_analyzerlib.nodes.entity.EntitySchema):
    def __init__(self):
        super(MachineSchema, self).__init__(
            default_machine_properties(), default_machine_edges(), lambda: MachineView
        )

    @staticmethod
    def self_type() -> str:
        return "Machine"


class MachineQuery(
    grapl_analyzerlib.nodes.entity.EntityQuery["MachineView", "MachineQuery"]
):
    def with_machine_id(
        self,
        *,
        eq: Optional[grapl_analyzerlib.comparators.StrOrNot] = None,
        contains: Optional[
            "grapl_analyzerlib.comparators.OneOrMany[grapl_analyzerlib.comparators.StrOrNot]"
        ] = None,
        starts_with: Optional["grapl_analyzerlib.comparators.StrOrNot"] = None,
        ends_with: Optional["grapl_analyzerlib.comparators.StrOrNot"] = None,
        regexp: Optional[
            "grapl_analyzerlib.comparators.OneOrMany[grapl_analyzerlib.comparators.StrOrNot]"
        ] = None,
        distance_lt: Optional[Tuple[str, int]] = None,
    ):
        (
            self.with_str_property(
                "machine_id",
                eq=eq,
                contains=contains,
                starts_with=starts_with,
                ends_with=ends_with,
                regexp=regexp,
                distance_lt=distance_lt,
            )
        )
        return self

    def with_hostname(
        self,
        *,
        eq: Optional[grapl_analyzerlib.comparators.StrOrNot] = None,
        contains: Optional[
            "grapl_analyzerlib.comparators.OneOrMany[grapl_analyzerlib.comparators.StrOrNot]"
        ] = None,
        starts_with: Optional["grapl_analyzerlib.comparators.StrOrNot"] = None,
        ends_with: Optional["grapl_analyzerlib.comparators.StrOrNot"] = None,
        regexp: Optional[
            "grapl_analyzerlib.comparators.OneOrMany[grapl_analyzerlib.comparators.StrOrNot]"
        ] = None,
        distance_lt: Optional[Tuple[str, int]] = None,
    ):
        (
            self.with_str_property(
                "hostname",
                eq=eq,
                contains=contains,
                starts_with=starts_with,
                ends_with=ends_with,
                regexp=regexp,
                distance_lt=distance_lt,
            )
        )
        return self

    def with_os(
        self,
        *,
        eq: Optional[grapl_analyzerlib.comparators.StrOrNot] = None,
        contains: Optional[
            "grapl_analyzerlib.comparators.OneOrMany[grapl_analyzerlib.comparators.StrOrNot]"
        ] = None,
        starts_with: Optional["grapl_analyzerlib.comparators.StrOrNot"] = None,
        ends_with: Optional["grapl_analyzerlib.comparators.StrOrNot"] = None,
        regexp: Optional[
            "grapl_analyzerlib.comparators.OneOrMany[grapl_analyzerlib.comparators.StrOrNot]"
        ] = None,
        distance_lt: Optional[Tuple[str, int]] = None,
    ):
        (
            self.with_str_property(
                "os",
                eq=eq,
                contains=contains,
                starts_with=starts_with,
                ends_with=ends_with,
                regexp=regexp,
                distance_lt=distance_lt,
            )
        )
        return self

    def with_machine_process(
        self: MachineQuery, *machine_process: ProcessQuery
    ) -> MachineQuery:
        return self.with_to_neighbor(
            MachineQuery,
            "machine_process",
            "process_executed_by_machine",
            machine_process,
        )

    def with_machine_files(
        self: MachineQuery, *machine_files: FileQuery
    ) -> MachineQuery:
        return self.with_to_neighbor(
            MachineQuery, "machine_files", "file_on_machine", machine_files
        )

    @classmethod
    def node_schema(cls) -> "grapl_analyzerlib.schema.Schema":
        return MachineSchema()


class MachineView(
    grapl_analyzerlib.nodes.entity.EntityView["MachineView", "MachineQuery"]
):
    queryable = MachineQuery

    def __init__(
        self,
        uid: int,
        node_key: str,
        graph_client: Any,
        node_types: Set[str],
        machine_id: Optional["str"] = None,
        hostname: Optional["str"] = None,
        os: Optional["str"] = None,
        machine_process: Optional[List["ProcessView"]] = None,
        machine_files: Optional[List["FileView"]] = None,
        **kwargs,
    ) -> None:
        super().__init__(uid, node_key, graph_client, node_types, **kwargs)
        if machine_id:
            self.set_predicate("machine_id", machine_id)
        if hostname:
            self.set_predicate("hostname", hostname)
        if os:
            self.set_predicate("os", os)
        if machine_process:
            self.set_predicate("machine_process", machine_process or [])
        if machine_files:
            self.set_predicate("machine_files", machine_files or [])

    def get_machine_id(self, cached: bool = True) -> Optional[str]:
        return self.get_str("machine_id", cached=cached)

    def get_hostname(self, cached: bool = True) -> Optional[str]:
        return self.get_str("hostname", cached=cached)

    def get_os(self, cached: bool = True) -> Optional[str]:
        return self.get_str("os", cached=cached)

    def get_machine_process(
        self, *machine_process: ProcessQuery, cached=False
    ) -> "List[ProcessView]":
        return self.get_neighbor(
            ProcessQuery,
            "machine_process",
            "process_executed_by_machine",
            machine_process,
            cached,
        )

    def get_machine_files(
        self, *machine_files: FileQuery, cached=False
    ) -> "List[FileView]":
        return self.get_neighbor(
            FileQuery, "machine_files", "file_on_machine", machine_files, cached
        )

    @classmethod
    def node_schema(cls) -> "grapl_analyzerlib.schema.Schema":
        return MachineSchema()


def default_networksocketaddress_properties() -> Dict[
    str, grapl_analyzerlib.node_types.PropType
]:
    return {
        "transport_protocol": grapl_analyzerlib.node_types.PropType(
            grapl_analyzerlib.node_types.PropPrimitive.Str, False
        ),
        "ip_address": grapl_analyzerlib.node_types.PropType(
            grapl_analyzerlib.node_types.PropPrimitive.Str, False
        ),
        "port_number": grapl_analyzerlib.node_types.PropType(
            grapl_analyzerlib.node_types.PropPrimitive.Str, False
        ),
    }


def default_networksocketaddress_edges() -> Dict[
    str, Tuple[grapl_analyzerlib.node_types.EdgeT, str]
]:
    return {
        "network_connection_to": (
            grapl_analyzerlib.node_types.EdgeT(
                NetworkSocketAddressSchema,
                NetworkSocketAddressSchema,
                grapl_analyzerlib.node_types.EdgeRelationship.ManyToMany,
            ),
            "network_connection_from",
        ),
        "network_connection_from": (
            grapl_analyzerlib.node_types.EdgeT(
                NetworkSocketAddressSchema,
                NetworkSocketAddressSchema,
                grapl_analyzerlib.node_types.EdgeRelationship.ManyToMany,
            ),
            "network_connection_to",
        ),
        "connection_from_process": (
            grapl_analyzerlib.node_types.EdgeT(
                NetworkSocketAddressSchema,
                ProcessSchema,
                grapl_analyzerlib.node_types.EdgeRelationship.ManyToMany,
            ),
            "process_connected_to",
        ),
        "connection_from_process_via": (
            grapl_analyzerlib.node_types.EdgeT(
                NetworkSocketAddressSchema,
                ProcessSchema,
                grapl_analyzerlib.node_types.EdgeRelationship.ManyToMany,
            ),
            "process_connected_via",
        ),
    }


class NetworkSocketAddressSchema(grapl_analyzerlib.nodes.entity.EntitySchema):
    def __init__(self):
        super(NetworkSocketAddressSchema, self).__init__(
            default_networksocketaddress_properties(),
            default_networksocketaddress_edges(),
            lambda: NetworkSocketAddressView,
        )

    @staticmethod
    def self_type() -> str:
        return "NetworkSocketAddress"


class NetworkSocketAddressQuery(
    grapl_analyzerlib.nodes.entity.EntityQuery[
        "NetworkSocketAddressView", "NetworkSocketAddressQuery"
    ]
):
    def with_transport_protocol(
        self,
        *,
        eq: Optional[grapl_analyzerlib.comparators.StrOrNot] = None,
        contains: Optional[
            "grapl_analyzerlib.comparators.OneOrMany[grapl_analyzerlib.comparators.StrOrNot]"
        ] = None,
        starts_with: Optional["grapl_analyzerlib.comparators.StrOrNot"] = None,
        ends_with: Optional["grapl_analyzerlib.comparators.StrOrNot"] = None,
        regexp: Optional[
            "grapl_analyzerlib.comparators.OneOrMany[grapl_analyzerlib.comparators.StrOrNot]"
        ] = None,
        distance_lt: Optional[Tuple[str, int]] = None,
    ):
        (
            self.with_str_property(
                "transport_protocol",
                eq=eq,
                contains=contains,
                starts_with=starts_with,
                ends_with=ends_with,
                regexp=regexp,
                distance_lt=distance_lt,
            )
        )
        return self

    def with_ip_address(
        self,
        *,
        eq: Optional[grapl_analyzerlib.comparators.StrOrNot] = None,
        contains: Optional[
            "grapl_analyzerlib.comparators.OneOrMany[grapl_analyzerlib.comparators.StrOrNot]"
        ] = None,
        starts_with: Optional["grapl_analyzerlib.comparators.StrOrNot"] = None,
        ends_with: Optional["grapl_analyzerlib.comparators.StrOrNot"] = None,
        regexp: Optional[
            "grapl_analyzerlib.comparators.OneOrMany[grapl_analyzerlib.comparators.StrOrNot]"
        ] = None,
        distance_lt: Optional[Tuple[str, int]] = None,
    ):
        (
            self.with_str_property(
                "ip_address",
                eq=eq,
                contains=contains,
                starts_with=starts_with,
                ends_with=ends_with,
                regexp=regexp,
                distance_lt=distance_lt,
            )
        )
        return self

    def with_port_number(
        self,
        *,
        eq: Optional[grapl_analyzerlib.comparators.StrOrNot] = None,
        contains: Optional[
            "grapl_analyzerlib.comparators.OneOrMany[grapl_analyzerlib.comparators.StrOrNot]"
        ] = None,
        starts_with: Optional["grapl_analyzerlib.comparators.StrOrNot"] = None,
        ends_with: Optional["grapl_analyzerlib.comparators.StrOrNot"] = None,
        regexp: Optional[
            "grapl_analyzerlib.comparators.OneOrMany[grapl_analyzerlib.comparators.StrOrNot]"
        ] = None,
        distance_lt: Optional[Tuple[str, int]] = None,
    ):
        (
            self.with_str_property(
                "port_number",
                eq=eq,
                contains=contains,
                starts_with=starts_with,
                ends_with=ends_with,
                regexp=regexp,
                distance_lt=distance_lt,
            )
        )
        return self

    def with_network_connection_to(
        self: NetworkSocketAddressQuery,
        *network_connection_to: NetworkSocketAddressQuery,
    ) -> NetworkSocketAddressQuery:
        return self.with_to_neighbor(
            NetworkSocketAddressQuery,
            "network_connection_to",
            "network_connection_from",
            network_connection_to,
        )

    def with_network_connection_from(
        self: NetworkSocketAddressQuery,
        *network_connection_from: NetworkSocketAddressQuery,
    ) -> NetworkSocketAddressQuery:
        return self.with_to_neighbor(
            NetworkSocketAddressQuery,
            "network_connection_from",
            "network_connection_to",
            network_connection_from,
        )

    def with_connection_from_process(
        self: NetworkSocketAddressQuery, *connection_from_process: ProcessQuery
    ) -> NetworkSocketAddressQuery:
        return self.with_to_neighbor(
            NetworkSocketAddressQuery,
            "connection_from_process",
            "process_connected_to",
            connection_from_process,
        )

    def with_connection_from_process_via(
        self: NetworkSocketAddressQuery, *connection_from_process_via: ProcessQuery
    ) -> NetworkSocketAddressQuery:
        return self.with_to_neighbor(
            NetworkSocketAddressQuery,
            "connection_from_process_via",
            "process_connected_via",
            connection_from_process_via,
        )

    @classmethod
    def node_schema(cls) -> "grapl_analyzerlib.schema.Schema":
        return NetworkSocketAddressSchema()


class NetworkSocketAddressView(
    grapl_analyzerlib.nodes.entity.EntityView[
        "NetworkSocketAddressView", "NetworkSocketAddressQuery"
    ]
):
    queryable = NetworkSocketAddressQuery

    def __init__(
        self,
        uid: int,
        node_key: str,
        graph_client: Any,
        node_types: Set[str],
        transport_protocol: Optional["str"] = None,
        ip_address: Optional["str"] = None,
        port_number: Optional["str"] = None,
        network_connection_to: Optional[List["NetworkSocketAddressView"]] = None,
        network_connection_from: Optional[List["NetworkSocketAddressView"]] = None,
        connection_from_process: Optional[List["ProcessView"]] = None,
        connection_from_process_via: Optional[List["ProcessView"]] = None,
        **kwargs,
    ) -> None:
        super().__init__(uid, node_key, graph_client, node_types, **kwargs)
        if transport_protocol:
            self.set_predicate("transport_protocol", transport_protocol)
        if ip_address:
            self.set_predicate("ip_address", ip_address)
        if port_number:
            self.set_predicate("port_number", port_number)
        if network_connection_to:
            self.set_predicate("network_connection_to", network_connection_to or [])
        if network_connection_from:
            self.set_predicate("network_connection_from", network_connection_from or [])
        if connection_from_process:
            self.set_predicate("connection_from_process", connection_from_process or [])
        if connection_from_process_via:
            self.set_predicate(
                "connection_from_process_via", connection_from_process_via or []
            )

    def get_transport_protocol(self, cached: bool = True) -> Optional[str]:
        return self.get_str("transport_protocol", cached=cached)

    def get_ip_address(self, cached: bool = True) -> Optional[str]:
        return self.get_str("ip_address", cached=cached)

    def get_port_number(self, cached: bool = True) -> Optional[str]:
        return self.get_str("port_number", cached=cached)

    def get_network_connection_to(
        self, *network_connection_to: NetworkSocketAddressQuery, cached=False
    ) -> "List[NetworkSocketAddressView]":
        return self.get_neighbor(
            NetworkSocketAddressQuery,
            "network_connection_to",
            "network_connection_from",
            network_connection_to,
            cached,
        )

    def get_network_connection_from(
        self, *network_connection_from: NetworkSocketAddressQuery, cached=False
    ) -> "List[NetworkSocketAddressView]":
        return self.get_neighbor(
            NetworkSocketAddressQuery,
            "network_connection_from",
            "network_connection_to",
            network_connection_from,
            cached,
        )

    def get_connection_from_process(
        self, *connection_from_process: ProcessQuery, cached=False
    ) -> "List[ProcessView]":
        return self.get_neighbor(
            ProcessQuery,
            "connection_from_process",
            "process_connected_to",
            connection_from_process,
            cached,
        )

    def get_connection_from_process_via(
        self, *connection_from_process_via: ProcessQuery, cached=False
    ) -> "List[ProcessView]":
        return self.get_neighbor(
            ProcessQuery,
            "connection_from_process_via",
            "process_connected_via",
            connection_from_process_via,
            cached,
        )

    @classmethod
    def node_schema(cls) -> "grapl_analyzerlib.schema.Schema":
        return NetworkSocketAddressSchema()


def default_process_properties() -> Dict[str, grapl_analyzerlib.node_types.PropType]:
    return {
        "pid": grapl_analyzerlib.node_types.PropType(
            grapl_analyzerlib.node_types.PropPrimitive.Int, False
        ),
        "guid": grapl_analyzerlib.node_types.PropType(
            grapl_analyzerlib.node_types.PropPrimitive.Str, False
        ),
        "created_timestamp": grapl_analyzerlib.node_types.PropType(
            grapl_analyzerlib.node_types.PropPrimitive.Int, False
        ),
        "cmdline": grapl_analyzerlib.node_types.PropType(
            grapl_analyzerlib.node_types.PropPrimitive.Str, False
        ),
        "image": grapl_analyzerlib.node_types.PropType(
            grapl_analyzerlib.node_types.PropPrimitive.Str, False
        ),
        "current_directory": grapl_analyzerlib.node_types.PropType(
            grapl_analyzerlib.node_types.PropPrimitive.Str, False
        ),
        "user": grapl_analyzerlib.node_types.PropType(
            grapl_analyzerlib.node_types.PropPrimitive.Str, False
        ),
    }


def default_process_edges() -> Dict[
    str, Tuple[grapl_analyzerlib.node_types.EdgeT, str]
]:
    return {
        "children": (
            grapl_analyzerlib.node_types.EdgeT(
                ProcessSchema,
                ProcessSchema,
                grapl_analyzerlib.node_types.EdgeRelationship.ManyToMany,
            ),
            "parent",
        ),
        "parent": (
            grapl_analyzerlib.node_types.EdgeT(
                ProcessSchema,
                ProcessSchema,
                grapl_analyzerlib.node_types.EdgeRelationship.ManyToMany,
            ),
            "children",
        ),
        "process_image": (
            grapl_analyzerlib.node_types.EdgeT(
                ProcessSchema,
                FileSchema,
                grapl_analyzerlib.node_types.EdgeRelationship.ManyToMany,
            ),
            "process_executed_from_image",
        ),
        "created_files": (
            grapl_analyzerlib.node_types.EdgeT(
                ProcessSchema,
                FileSchema,
                grapl_analyzerlib.node_types.EdgeRelationship.ManyToMany,
            ),
            "created_by_process",
        ),
        "process_executed_by_machine": (
            grapl_analyzerlib.node_types.EdgeT(
                ProcessSchema,
                MachineSchema,
                grapl_analyzerlib.node_types.EdgeRelationship.ManyToMany,
            ),
            "machine_process",
        ),
        "process_connected_to": (
            grapl_analyzerlib.node_types.EdgeT(
                ProcessSchema,
                NetworkSocketAddressSchema,
                grapl_analyzerlib.node_types.EdgeRelationship.ManyToMany,
            ),
            "connection_from_process",
        ),
        "process_connected_via": (
            grapl_analyzerlib.node_types.EdgeT(
                ProcessSchema,
                NetworkSocketAddressSchema,
                grapl_analyzerlib.node_types.EdgeRelationship.ManyToMany,
            ),
            "connection_from_process_via",
        ),
    }


class ProcessSchema(grapl_analyzerlib.nodes.entity.EntitySchema):
    def __init__(self):
        super(ProcessSchema, self).__init__(
            default_process_properties(), default_process_edges(), lambda: ProcessView
        )

    @staticmethod
    def self_type() -> str:
        return "Process"


class ProcessQuery(
    grapl_analyzerlib.nodes.entity.EntityQuery["ProcessView", "ProcessQuery"]
):
    def with_pid(
        self,
        *,
        eq: Optional["grapl_analyzerlib.comparators.IntOrNot"] = None,
        gt: Optional["grapl_analyzerlib.comparators.IntOrNot"] = None,
        ge: Optional["grapl_analyzerlib.comparators.IntOrNot"] = None,
        lt: Optional["grapl_analyzerlib.comparators.IntOrNot"] = None,
        le: Optional["grapl_analyzerlib.comparators.IntOrNot"] = None,
    ):
        (
            self.with_int_property(
                "pid",
                eq=eq,
                gt=gt,
                ge=ge,
                lt=lt,
                le=le,
            )
        )
        return self

    def with_guid(
        self,
        *,
        eq: Optional[grapl_analyzerlib.comparators.StrOrNot] = None,
        contains: Optional[
            "grapl_analyzerlib.comparators.OneOrMany[grapl_analyzerlib.comparators.StrOrNot]"
        ] = None,
        starts_with: Optional["grapl_analyzerlib.comparators.StrOrNot"] = None,
        ends_with: Optional["grapl_analyzerlib.comparators.StrOrNot"] = None,
        regexp: Optional[
            "grapl_analyzerlib.comparators.OneOrMany[grapl_analyzerlib.comparators.StrOrNot]"
        ] = None,
        distance_lt: Optional[Tuple[str, int]] = None,
    ):
        (
            self.with_str_property(
                "guid",
                eq=eq,
                contains=contains,
                starts_with=starts_with,
                ends_with=ends_with,
                regexp=regexp,
                distance_lt=distance_lt,
            )
        )
        return self

    def with_created_timestamp(
        self,
        *,
        eq: Optional["grapl_analyzerlib.comparators.IntOrNot"] = None,
        gt: Optional["grapl_analyzerlib.comparators.IntOrNot"] = None,
        ge: Optional["grapl_analyzerlib.comparators.IntOrNot"] = None,
        lt: Optional["grapl_analyzerlib.comparators.IntOrNot"] = None,
        le: Optional["grapl_analyzerlib.comparators.IntOrNot"] = None,
    ):
        (
            self.with_int_property(
                "created_timestamp",
                eq=eq,
                gt=gt,
                ge=ge,
                lt=lt,
                le=le,
            )
        )
        return self

    def with_cmdline(
        self,
        *,
        eq: Optional[grapl_analyzerlib.comparators.StrOrNot] = None,
        contains: Optional[
            "grapl_analyzerlib.comparators.OneOrMany[grapl_analyzerlib.comparators.StrOrNot]"
        ] = None,
        starts_with: Optional["grapl_analyzerlib.comparators.StrOrNot"] = None,
        ends_with: Optional["grapl_analyzerlib.comparators.StrOrNot"] = None,
        regexp: Optional[
            "grapl_analyzerlib.comparators.OneOrMany[grapl_analyzerlib.comparators.StrOrNot]"
        ] = None,
        distance_lt: Optional[Tuple[str, int]] = None,
    ):
        (
            self.with_str_property(
                "cmdline",
                eq=eq,
                contains=contains,
                starts_with=starts_with,
                ends_with=ends_with,
                regexp=regexp,
                distance_lt=distance_lt,
            )
        )
        return self

    def with_image(
        self,
        *,
        eq: Optional[grapl_analyzerlib.comparators.StrOrNot] = None,
        contains: Optional[
            "grapl_analyzerlib.comparators.OneOrMany[grapl_analyzerlib.comparators.StrOrNot]"
        ] = None,
        starts_with: Optional["grapl_analyzerlib.comparators.StrOrNot"] = None,
        ends_with: Optional["grapl_analyzerlib.comparators.StrOrNot"] = None,
        regexp: Optional[
            "grapl_analyzerlib.comparators.OneOrMany[grapl_analyzerlib.comparators.StrOrNot]"
        ] = None,
        distance_lt: Optional[Tuple[str, int]] = None,
    ):
        (
            self.with_str_property(
                "image",
                eq=eq,
                contains=contains,
                starts_with=starts_with,
                ends_with=ends_with,
                regexp=regexp,
                distance_lt=distance_lt,
            )
        )
        return self

    def with_current_directory(
        self,
        *,
        eq: Optional[grapl_analyzerlib.comparators.StrOrNot] = None,
        contains: Optional[
            "grapl_analyzerlib.comparators.OneOrMany[grapl_analyzerlib.comparators.StrOrNot]"
        ] = None,
        starts_with: Optional["grapl_analyzerlib.comparators.StrOrNot"] = None,
        ends_with: Optional["grapl_analyzerlib.comparators.StrOrNot"] = None,
        regexp: Optional[
            "grapl_analyzerlib.comparators.OneOrMany[grapl_analyzerlib.comparators.StrOrNot]"
        ] = None,
        distance_lt: Optional[Tuple[str, int]] = None,
    ):
        (
            self.with_str_property(
                "current_directory",
                eq=eq,
                contains=contains,
                starts_with=starts_with,
                ends_with=ends_with,
                regexp=regexp,
                distance_lt=distance_lt,
            )
        )
        return self

    def with_user(
        self,
        *,
        eq: Optional[grapl_analyzerlib.comparators.StrOrNot] = None,
        contains: Optional[
            "grapl_analyzerlib.comparators.OneOrMany[grapl_analyzerlib.comparators.StrOrNot]"
        ] = None,
        starts_with: Optional["grapl_analyzerlib.comparators.StrOrNot"] = None,
        ends_with: Optional["grapl_analyzerlib.comparators.StrOrNot"] = None,
        regexp: Optional[
            "grapl_analyzerlib.comparators.OneOrMany[grapl_analyzerlib.comparators.StrOrNot]"
        ] = None,
        distance_lt: Optional[Tuple[str, int]] = None,
    ):
        (
            self.with_str_property(
                "user",
                eq=eq,
                contains=contains,
                starts_with=starts_with,
                ends_with=ends_with,
                regexp=regexp,
                distance_lt=distance_lt,
            )
        )
        return self

    def with_children(self: ProcessQuery, *children: ProcessQuery) -> ProcessQuery:
        return self.with_to_neighbor(ProcessQuery, "children", "parent", children)

    def with_parent(self: ProcessQuery, *parent: ProcessQuery) -> ProcessQuery:
        return self.with_to_neighbor(ProcessQuery, "parent", "children", parent)

    def with_process_image(
        self: ProcessQuery, *process_image: FileQuery
    ) -> ProcessQuery:
        return self.with_to_neighbor(
            ProcessQuery, "process_image", "process_executed_from_image", process_image
        )

    def with_created_files(
        self: ProcessQuery, *created_files: FileQuery
    ) -> ProcessQuery:
        return self.with_to_neighbor(
            ProcessQuery, "created_files", "created_by_process", created_files
        )

    def with_process_executed_by_machine(
        self: ProcessQuery, *process_executed_by_machine: MachineQuery
    ) -> ProcessQuery:
        return self.with_to_neighbor(
            ProcessQuery,
            "process_executed_by_machine",
            "machine_process",
            process_executed_by_machine,
        )

    def with_process_connected_to(
        self: ProcessQuery, *process_connected_to: NetworkSocketAddressQuery
    ) -> ProcessQuery:
        return self.with_to_neighbor(
            ProcessQuery,
            "process_connected_to",
            "connection_from_process",
            process_connected_to,
        )

    def with_process_connected_via(
        self: ProcessQuery, *process_connected_via: NetworkSocketAddressQuery
    ) -> ProcessQuery:
        return self.with_to_neighbor(
            ProcessQuery,
            "process_connected_via",
            "connection_from_process_via",
            process_connected_via,
        )

    @classmethod
    def node_schema(cls) -> "grapl_analyzerlib.schema.Schema":
        return ProcessSchema()


class ProcessView(
    grapl_analyzerlib.nodes.entity.EntityView["ProcessView", "ProcessQuery"]
):
    queryable = ProcessQuery

    def __init__(
        self,
        uid: int,
        node_key: str,
        graph_client: Any,
        node_types: Set[str],
        pid: Optional["int"] = None,
        guid: Optional["str"] = None,
        created_timestamp: Optional["int"] = None,
        cmdline: Optional["str"] = None,
        image: Optional["str"] = None,
        current_directory: Optional["str"] = None,
        user: Optional["str"] = None,
        children: Optional[List["ProcessView"]] = None,
        parent: Optional[List["ProcessView"]] = None,
        process_image: Optional[List["FileView"]] = None,
        created_files: Optional[List["FileView"]] = None,
        process_executed_by_machine: Optional[List["MachineView"]] = None,
        process_connected_to: Optional[List["NetworkSocketAddressView"]] = None,
        process_connected_via: Optional[List["NetworkSocketAddressView"]] = None,
        **kwargs,
    ) -> None:
        super().__init__(uid, node_key, graph_client, node_types, **kwargs)
        if pid:
            self.set_predicate("pid", pid)
        if guid:
            self.set_predicate("guid", guid)
        if created_timestamp:
            self.set_predicate("created_timestamp", created_timestamp)
        if cmdline:
            self.set_predicate("cmdline", cmdline)
        if image:
            self.set_predicate("image", image)
        if current_directory:
            self.set_predicate("current_directory", current_directory)
        if user:
            self.set_predicate("user", user)
        if children:
            self.set_predicate("children", children or [])
        if parent:
            self.set_predicate("parent", parent or [])
        if process_image:
            self.set_predicate("process_image", process_image or [])
        if created_files:
            self.set_predicate("created_files", created_files or [])
        if process_executed_by_machine:
            self.set_predicate(
                "process_executed_by_machine", process_executed_by_machine or []
            )
        if process_connected_to:
            self.set_predicate("process_connected_to", process_connected_to or [])
        if process_connected_via:
            self.set_predicate("process_connected_via", process_connected_via or [])

    def get_pid(self, cached: bool = True) -> Optional[int]:
        return self.get_int("pid", cached=cached)

    def get_guid(self, cached: bool = True) -> Optional[str]:
        return self.get_str("guid", cached=cached)

    def get_created_timestamp(self, cached: bool = True) -> Optional[int]:
        return self.get_int("created_timestamp", cached=cached)

    def get_cmdline(self, cached: bool = True) -> Optional[str]:
        return self.get_str("cmdline", cached=cached)

    def get_image(self, cached: bool = True) -> Optional[str]:
        return self.get_str("image", cached=cached)

    def get_current_directory(self, cached: bool = True) -> Optional[str]:
        return self.get_str("current_directory", cached=cached)

    def get_user(self, cached: bool = True) -> Optional[str]:
        return self.get_str("user", cached=cached)

    def get_children(
        self, *children: ProcessQuery, cached=False
    ) -> "List[ProcessView]":
        return self.get_neighbor(ProcessQuery, "children", "parent", children, cached)

    def get_parent(self, *parent: ProcessQuery, cached=False) -> "List[ProcessView]":
        return self.get_neighbor(ProcessQuery, "parent", "children", parent, cached)

    def get_process_image(
        self, *process_image: FileQuery, cached=False
    ) -> "List[FileView]":
        return self.get_neighbor(
            FileQuery,
            "process_image",
            "process_executed_from_image",
            process_image,
            cached,
        )

    def get_created_files(
        self, *created_files: FileQuery, cached=False
    ) -> "List[FileView]":
        return self.get_neighbor(
            FileQuery, "created_files", "created_by_process", created_files, cached
        )

    def get_process_executed_by_machine(
        self, *process_executed_by_machine: MachineQuery, cached=False
    ) -> "List[MachineView]":
        return self.get_neighbor(
            MachineQuery,
            "process_executed_by_machine",
            "machine_process",
            process_executed_by_machine,
            cached,
        )

    def get_process_connected_to(
        self, *process_connected_to: NetworkSocketAddressQuery, cached=False
    ) -> "List[NetworkSocketAddressView]":
        return self.get_neighbor(
            NetworkSocketAddressQuery,
            "process_connected_to",
            "connection_from_process",
            process_connected_to,
            cached,
        )

    def get_process_connected_via(
        self, *process_connected_via: NetworkSocketAddressQuery, cached=False
    ) -> "List[NetworkSocketAddressView]":
        return self.get_neighbor(
            NetworkSocketAddressQuery,
            "process_connected_via",
            "connection_from_process_via",
            process_connected_via,
            cached,
        )

    @classmethod
    def node_schema(cls) -> "grapl_analyzerlib.schema.Schema":
        return ProcessSchema()
