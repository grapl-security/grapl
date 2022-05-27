from __future__ import annotations
from typing import Optional, Any, Set, List, Dict, Tuple
import grapl_analyzerlib
import grapl_analyzerlib.node_types
import grapl_analyzerlib.nodes.entity
import grapl_analyzerlib.queryable
def default_asset_properties() -> Dict[str, grapl_analyzerlib.node_types.PropType]:
    return {
        "asset_id": grapl_analyzerlib.node_types.PropType(grapl_analyzerlib.node_types.PropPrimitive.Str, False),
        "hostname": grapl_analyzerlib.node_types.PropType(grapl_analyzerlib.node_types.PropPrimitive.Str, False),
        "os": grapl_analyzerlib.node_types.PropType(grapl_analyzerlib.node_types.PropPrimitive.Str, False),
    }
def default_asset_edges() -> Dict[str, Tuple[grapl_analyzerlib.node_types.EdgeT, str]]:
    return {
        "asset_processes": (
            grapl_analyzerlib.node_types.EdgeT(AssetSchema, ProcessSchema, grapl_analyzerlib.node_types.EdgeRelationship.ManyToMany),
            "process_executed_by_asset"
        ),
        "asset_files": (
            grapl_analyzerlib.node_types.EdgeT(AssetSchema, FileSchema, grapl_analyzerlib.node_types.EdgeRelationship.ManyToMany),
            "file_on_asset"
        ),
    }

class AssetSchema(grapl_analyzerlib.nodes.entity.EntitySchema):
    def __init__(self):
        super(AssetSchema, self).__init__(
            default_asset_properties(), default_asset_edges(), lambda: AssetView
        );

    @staticmethod
    def self_type() -> str:
        return "Asset"



class AssetQuery(grapl_analyzerlib.nodes.entity.EntityQuery['AssetView', 'AssetQuery']):
    def with_asset_id(
        self,
        *,
        eq: Optional[grapl_analyzerlib.comparators.StrOrNot] = None,
        contains: Optional["grapl_analyzerlib.comparators.OneOrMany[grapl_analyzerlib.comparators.StrOrNot]"] = None,
        starts_with: Optional["grapl_analyzerlib.comparators.StrOrNot"] = None,
        ends_with: Optional["grapl_analyzerlib.comparators.StrOrNot"] = None,
        regexp: Optional["grapl_analyzerlib.comparators.OneOrMany[grapl_analyzerlib.comparators.StrOrNot]"] = None,
        distance_lt: Optional[Tuple[str, int]] = None,
    ):
        (
            self.with_str_property(
                "asset_id",
                eq=eq,
                contains=contains,
                starts_with=starts_with,
                ends_with=ends_with,
                regexp=regexp,
                distance_lt=distance_lt
            )
        )
        return self

    def with_hostname(
        self,
        *,
        eq: Optional[grapl_analyzerlib.comparators.StrOrNot] = None,
        contains: Optional["grapl_analyzerlib.comparators.OneOrMany[grapl_analyzerlib.comparators.StrOrNot]"] = None,
        starts_with: Optional["grapl_analyzerlib.comparators.StrOrNot"] = None,
        ends_with: Optional["grapl_analyzerlib.comparators.StrOrNot"] = None,
        regexp: Optional["grapl_analyzerlib.comparators.OneOrMany[grapl_analyzerlib.comparators.StrOrNot]"] = None,
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
                distance_lt=distance_lt
            )
        )
        return self

    def with_os(
        self,
        *,
        eq: Optional[grapl_analyzerlib.comparators.StrOrNot] = None,
        contains: Optional["grapl_analyzerlib.comparators.OneOrMany[grapl_analyzerlib.comparators.StrOrNot]"] = None,
        starts_with: Optional["grapl_analyzerlib.comparators.StrOrNot"] = None,
        ends_with: Optional["grapl_analyzerlib.comparators.StrOrNot"] = None,
        regexp: Optional["grapl_analyzerlib.comparators.OneOrMany[grapl_analyzerlib.comparators.StrOrNot]"] = None,
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
                distance_lt=distance_lt
            )
        )
        return self

    def with_asset_processes(self: AssetQuery, *asset_processes: ProcessQuery) -> AssetQuery:
        return self.with_to_neighbor(AssetQuery, "asset_processes", "process_executed_by_asset", asset_processes)

    def with_asset_files(self: AssetQuery, *asset_files: FileQuery) -> AssetQuery:
        return self.with_to_neighbor(AssetQuery, "asset_files", "file_on_asset", asset_files)

    @classmethod
    def node_schema(cls) -> "grapl_analyzerlib.schema.Schema":
        return AssetSchema()


class AssetView(grapl_analyzerlib.nodes.entity.EntityView['AssetView', 'AssetQuery']):
    queryable = AssetQuery

    def __init__(
        self,
        uid: int,
        node_key: str,
        graph_client: Any,
        node_types: Set[str],
        asset_id: Optional["str"] = None,
        hostname: Optional["str"] = None,
        os: Optional["str"] = None,
        asset_processes: Optional[List["ProcessView"]] = None,
        asset_files: Optional[List["FileView"]] = None,
        **kwargs,
    ) -> None:
        super().__init__(uid, node_key, graph_client, node_types, **kwargs)
        if asset_id: self.set_predicate("asset_id", asset_id)
        if hostname: self.set_predicate("hostname", hostname)
        if os: self.set_predicate("os", os)
        if asset_processes: self.set_predicate("asset_processes", asset_processes or [])
        if asset_files: self.set_predicate("asset_files", asset_files or [])

    def get_asset_id(self, cached: bool = True) -> Optional[str]:
        return self.get_str("asset_id", cached=cached)

    def get_hostname(self, cached: bool = True) -> Optional[str]:
        return self.get_str("hostname", cached=cached)

    def get_os(self, cached: bool = True) -> Optional[str]:
        return self.get_str("os", cached=cached)

    def get_asset_processes(self, *asset_processes: ProcessQuery, cached=False) -> 'List[ProcessView]':
          return self.get_neighbor(ProcessQuery, "asset_processes", "process_executed_by_asset", asset_processes, cached)
    def get_asset_files(self, *asset_files: FileQuery, cached=False) -> 'List[FileView]':
          return self.get_neighbor(FileQuery, "asset_files", "file_on_asset", asset_files, cached)

    @classmethod
    def node_schema(cls) -> "grapl_analyzerlib.schema.Schema":
        return AssetSchema()

def default_file_properties() -> Dict[str, grapl_analyzerlib.node_types.PropType]:
    return {
        "asset_id": grapl_analyzerlib.node_types.PropType(grapl_analyzerlib.node_types.PropPrimitive.Str, False),
        "path": grapl_analyzerlib.node_types.PropType(grapl_analyzerlib.node_types.PropPrimitive.Str, False),
    }
def default_file_edges() -> Dict[str, Tuple[grapl_analyzerlib.node_types.EdgeT, str]]:
    return {
        "process_executed_from_exe": (
            grapl_analyzerlib.node_types.EdgeT(FileSchema, ProcessSchema, grapl_analyzerlib.node_types.EdgeRelationship.ManyToMany),
            "process_exe"
        ),
        "created_by_process": (
            grapl_analyzerlib.node_types.EdgeT(FileSchema, ProcessSchema, grapl_analyzerlib.node_types.EdgeRelationship.ManyToMany),
            "created_files"
        ),
        "file_on_asset": (
            grapl_analyzerlib.node_types.EdgeT(FileSchema, AssetSchema, grapl_analyzerlib.node_types.EdgeRelationship.ManyToMany),
            "asset_files"
        ),
    }

class FileSchema(grapl_analyzerlib.nodes.entity.EntitySchema):
    def __init__(self):
        super(FileSchema, self).__init__(
            default_file_properties(), default_file_edges(), lambda: FileView
        );

    @staticmethod
    def self_type() -> str:
        return "File"



class FileQuery(grapl_analyzerlib.nodes.entity.EntityQuery['FileView', 'FileQuery']):
    def with_asset_id(
        self,
        *,
        eq: Optional[grapl_analyzerlib.comparators.StrOrNot] = None,
        contains: Optional["grapl_analyzerlib.comparators.OneOrMany[grapl_analyzerlib.comparators.StrOrNot]"] = None,
        starts_with: Optional["grapl_analyzerlib.comparators.StrOrNot"] = None,
        ends_with: Optional["grapl_analyzerlib.comparators.StrOrNot"] = None,
        regexp: Optional["grapl_analyzerlib.comparators.OneOrMany[grapl_analyzerlib.comparators.StrOrNot]"] = None,
        distance_lt: Optional[Tuple[str, int]] = None,
    ):
        (
            self.with_str_property(
                "asset_id",
                eq=eq,
                contains=contains,
                starts_with=starts_with,
                ends_with=ends_with,
                regexp=regexp,
                distance_lt=distance_lt
            )
        )
        return self

    def with_path(
        self,
        *,
        eq: Optional[grapl_analyzerlib.comparators.StrOrNot] = None,
        contains: Optional["grapl_analyzerlib.comparators.OneOrMany[grapl_analyzerlib.comparators.StrOrNot]"] = None,
        starts_with: Optional["grapl_analyzerlib.comparators.StrOrNot"] = None,
        ends_with: Optional["grapl_analyzerlib.comparators.StrOrNot"] = None,
        regexp: Optional["grapl_analyzerlib.comparators.OneOrMany[grapl_analyzerlib.comparators.StrOrNot]"] = None,
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
                distance_lt=distance_lt
            )
        )
        return self

    def with_process_executed_from_exe(self: FileQuery, *process_executed_from_exe: ProcessQuery) -> FileQuery:
        return self.with_to_neighbor(FileQuery, "process_executed_from_exe", "process_exe", process_executed_from_exe)

    def with_created_by_process(self: FileQuery, *created_by_process: ProcessQuery) -> FileQuery:
        return self.with_to_neighbor(FileQuery, "created_by_process", "created_files", created_by_process)

    def with_file_on_asset(self: FileQuery, *file_on_asset: AssetQuery) -> FileQuery:
        return self.with_to_neighbor(FileQuery, "file_on_asset", "asset_files", file_on_asset)

    @classmethod
    def node_schema(cls) -> "grapl_analyzerlib.schema.Schema":
        return FileSchema()


class FileView(grapl_analyzerlib.nodes.entity.EntityView['FileView', 'FileQuery']):
    queryable = FileQuery

    def __init__(
        self,
        uid: int,
        node_key: str,
        graph_client: Any,
        node_types: Set[str],
        asset_id: Optional["str"] = None,
        path: Optional["str"] = None,
        process_executed_from_exe: Optional[List["ProcessView"]] = None,
        created_by_process: Optional[List["ProcessView"]] = None,
        file_on_asset: Optional[List["AssetView"]] = None,
        **kwargs,
    ) -> None:
        super().__init__(uid, node_key, graph_client, node_types, **kwargs)
        if asset_id: self.set_predicate("asset_id", asset_id)
        if path: self.set_predicate("path", path)
        if process_executed_from_exe: self.set_predicate("process_executed_from_exe", process_executed_from_exe or [])
        if created_by_process: self.set_predicate("created_by_process", created_by_process or [])
        if file_on_asset: self.set_predicate("file_on_asset", file_on_asset or [])

    def get_asset_id(self, cached: bool = True) -> Optional[str]:
        return self.get_str("asset_id", cached=cached)

    def get_path(self, cached: bool = True) -> Optional[str]:
        return self.get_str("path", cached=cached)

    def get_process_executed_from_exe(self, *process_executed_from_exe: ProcessQuery, cached=False) -> 'List[ProcessView]':
          return self.get_neighbor(ProcessQuery, "process_executed_from_exe", "process_exe", process_executed_from_exe, cached)
    def get_created_by_process(self, *created_by_process: ProcessQuery, cached=False) -> 'List[ProcessView]':
          return self.get_neighbor(ProcessQuery, "created_by_process", "created_files", created_by_process, cached)
    def get_file_on_asset(self, *file_on_asset: AssetQuery, cached=False) -> 'List[AssetView]':
          return self.get_neighbor(AssetQuery, "file_on_asset", "asset_files", file_on_asset, cached)

    @classmethod
    def node_schema(cls) -> "grapl_analyzerlib.schema.Schema":
        return FileSchema()

def default_ipv4address_properties() -> Dict[str, grapl_analyzerlib.node_types.PropType]:
    return {
        "address": grapl_analyzerlib.node_types.PropType(grapl_analyzerlib.node_types.PropPrimitive.Str, False),
    }
def default_ipv4address_edges() -> Dict[str, Tuple[grapl_analyzerlib.node_types.EdgeT, str]]:
    return {
        "network_sockets_ipv4": (
            grapl_analyzerlib.node_types.EdgeT(IpV4AddressSchema, NetworkSocketAddressSchema, grapl_analyzerlib.node_types.EdgeRelationship.ManyToMany),
            "socket_ipv4_address"
        ),
    }

class IpV4AddressSchema(grapl_analyzerlib.nodes.entity.EntitySchema):
    def __init__(self):
        super(IpV4AddressSchema, self).__init__(
            default_ipv4address_properties(), default_ipv4address_edges(), lambda: IpV4AddressView
        );

    @staticmethod
    def self_type() -> str:
        return "IpV4Address"



class IpV4AddressQuery(grapl_analyzerlib.nodes.entity.EntityQuery['IpV4AddressView', 'IpV4AddressQuery']):
    def with_address(
        self,
        *,
        eq: Optional[grapl_analyzerlib.comparators.StrOrNot] = None,
        contains: Optional["grapl_analyzerlib.comparators.OneOrMany[grapl_analyzerlib.comparators.StrOrNot]"] = None,
        starts_with: Optional["grapl_analyzerlib.comparators.StrOrNot"] = None,
        ends_with: Optional["grapl_analyzerlib.comparators.StrOrNot"] = None,
        regexp: Optional["grapl_analyzerlib.comparators.OneOrMany[grapl_analyzerlib.comparators.StrOrNot]"] = None,
        distance_lt: Optional[Tuple[str, int]] = None,
    ):
        (
            self.with_str_property(
                "address",
                eq=eq,
                contains=contains,
                starts_with=starts_with,
                ends_with=ends_with,
                regexp=regexp,
                distance_lt=distance_lt
            )
        )
        return self

    def with_network_sockets_ipv4(self: IpV4AddressQuery, *network_sockets_ipv4: NetworkSocketAddressQuery) -> IpV4AddressQuery:
        return self.with_to_neighbor(IpV4AddressQuery, "network_sockets_ipv4", "socket_ipv4_address", network_sockets_ipv4)

    @classmethod
    def node_schema(cls) -> "grapl_analyzerlib.schema.Schema":
        return IpV4AddressSchema()


class IpV4AddressView(grapl_analyzerlib.nodes.entity.EntityView['IpV4AddressView', 'IpV4AddressQuery']):
    queryable = IpV4AddressQuery

    def __init__(
        self,
        uid: int,
        node_key: str,
        graph_client: Any,
        node_types: Set[str],
        address: Optional["str"] = None,
        network_sockets_ipv4: Optional[List["NetworkSocketAddressView"]] = None,
        **kwargs,
    ) -> None:
        super().__init__(uid, node_key, graph_client, node_types, **kwargs)
        if address: self.set_predicate("address", address)
        if network_sockets_ipv4: self.set_predicate("network_sockets_ipv4", network_sockets_ipv4 or [])

    def get_address(self, cached: bool = True) -> Optional[str]:
        return self.get_str("address", cached=cached)

    def get_network_sockets_ipv4(self, *network_sockets_ipv4: NetworkSocketAddressQuery, cached=False) -> 'List[NetworkSocketAddressView]':
          return self.get_neighbor(NetworkSocketAddressQuery, "network_sockets_ipv4", "socket_ipv4_address", network_sockets_ipv4, cached)

    @classmethod
    def node_schema(cls) -> "grapl_analyzerlib.schema.Schema":
        return IpV4AddressSchema()

def default_ipv6address_properties() -> Dict[str, grapl_analyzerlib.node_types.PropType]:
    return {
        "address": grapl_analyzerlib.node_types.PropType(grapl_analyzerlib.node_types.PropPrimitive.Str, False),
    }
def default_ipv6address_edges() -> Dict[str, Tuple[grapl_analyzerlib.node_types.EdgeT, str]]:
    return {
        "network_sockets_ipv6": (
            grapl_analyzerlib.node_types.EdgeT(IpV6AddressSchema, NetworkSocketAddressSchema, grapl_analyzerlib.node_types.EdgeRelationship.ManyToMany),
            "socket_ipv6_address"
        ),
    }

class IpV6AddressSchema(grapl_analyzerlib.nodes.entity.EntitySchema):
    def __init__(self):
        super(IpV6AddressSchema, self).__init__(
            default_ipv6address_properties(), default_ipv6address_edges(), lambda: IpV6AddressView
        );

    @staticmethod
    def self_type() -> str:
        return "IpV6Address"



class IpV6AddressQuery(grapl_analyzerlib.nodes.entity.EntityQuery['IpV6AddressView', 'IpV6AddressQuery']):
    def with_address(
        self,
        *,
        eq: Optional[grapl_analyzerlib.comparators.StrOrNot] = None,
        contains: Optional["grapl_analyzerlib.comparators.OneOrMany[grapl_analyzerlib.comparators.StrOrNot]"] = None,
        starts_with: Optional["grapl_analyzerlib.comparators.StrOrNot"] = None,
        ends_with: Optional["grapl_analyzerlib.comparators.StrOrNot"] = None,
        regexp: Optional["grapl_analyzerlib.comparators.OneOrMany[grapl_analyzerlib.comparators.StrOrNot]"] = None,
        distance_lt: Optional[Tuple[str, int]] = None,
    ):
        (
            self.with_str_property(
                "address",
                eq=eq,
                contains=contains,
                starts_with=starts_with,
                ends_with=ends_with,
                regexp=regexp,
                distance_lt=distance_lt
            )
        )
        return self

    def with_network_sockets_ipv6(self: IpV6AddressQuery, *network_sockets_ipv6: NetworkSocketAddressQuery) -> IpV6AddressQuery:
        return self.with_to_neighbor(IpV6AddressQuery, "network_sockets_ipv6", "socket_ipv6_address", network_sockets_ipv6)

    @classmethod
    def node_schema(cls) -> "grapl_analyzerlib.schema.Schema":
        return IpV6AddressSchema()


class IpV6AddressView(grapl_analyzerlib.nodes.entity.EntityView['IpV6AddressView', 'IpV6AddressQuery']):
    queryable = IpV6AddressQuery

    def __init__(
        self,
        uid: int,
        node_key: str,
        graph_client: Any,
        node_types: Set[str],
        address: Optional["str"] = None,
        network_sockets_ipv6: Optional[List["NetworkSocketAddressView"]] = None,
        **kwargs,
    ) -> None:
        super().__init__(uid, node_key, graph_client, node_types, **kwargs)
        if address: self.set_predicate("address", address)
        if network_sockets_ipv6: self.set_predicate("network_sockets_ipv6", network_sockets_ipv6 or [])

    def get_address(self, cached: bool = True) -> Optional[str]:
        return self.get_str("address", cached=cached)

    def get_network_sockets_ipv6(self, *network_sockets_ipv6: NetworkSocketAddressQuery, cached=False) -> 'List[NetworkSocketAddressView]':
          return self.get_neighbor(NetworkSocketAddressQuery, "network_sockets_ipv6", "socket_ipv6_address", network_sockets_ipv6, cached)

    @classmethod
    def node_schema(cls) -> "grapl_analyzerlib.schema.Schema":
        return IpV6AddressSchema()

def default_networksocketaddress_properties() -> Dict[str, grapl_analyzerlib.node_types.PropType]:
    return {
        "transport_protocol": grapl_analyzerlib.node_types.PropType(grapl_analyzerlib.node_types.PropPrimitive.Str, False),
        "port_number": grapl_analyzerlib.node_types.PropType(grapl_analyzerlib.node_types.PropPrimitive.Int, False),
        "ip_address": grapl_analyzerlib.node_types.PropType(grapl_analyzerlib.node_types.PropPrimitive.Str, False),
    }
def default_networksocketaddress_edges() -> Dict[str, Tuple[grapl_analyzerlib.node_types.EdgeT, str]]:
    return {
        "socket_ipv4_address": (
            grapl_analyzerlib.node_types.EdgeT(NetworkSocketAddressSchema, IpV4AddressSchema, grapl_analyzerlib.node_types.EdgeRelationship.ManyToMany),
            "network_sockets_ipv4"
        ),
        "socket_ipv6_address": (
            grapl_analyzerlib.node_types.EdgeT(NetworkSocketAddressSchema, IpV6AddressSchema, grapl_analyzerlib.node_types.EdgeRelationship.ManyToMany),
            "network_sockets_ipv6"
        ),
        "tcp_connection_to_a": (
            grapl_analyzerlib.node_types.EdgeT(NetworkSocketAddressSchema, TcpConnectionSchema, grapl_analyzerlib.node_types.EdgeRelationship.ManyToMany),
            "tcp_connection_from_b"
        ),
        "tcp_connection_from_a": (
            grapl_analyzerlib.node_types.EdgeT(NetworkSocketAddressSchema, TcpConnectionSchema, grapl_analyzerlib.node_types.EdgeRelationship.ManyToMany),
            "tcp_connection_to_b"
        ),
        "socket_process_outbound": (
            grapl_analyzerlib.node_types.EdgeT(NetworkSocketAddressSchema, ProcessSchema, grapl_analyzerlib.node_types.EdgeRelationship.ManyToMany),
            "process_socket_outbound"
        ),
        "socket_process_inbound": (
            grapl_analyzerlib.node_types.EdgeT(NetworkSocketAddressSchema, ProcessSchema, grapl_analyzerlib.node_types.EdgeRelationship.ManyToMany),
            "process_socket_inbound"
        ),
    }

class NetworkSocketAddressSchema(grapl_analyzerlib.nodes.entity.EntitySchema):
    def __init__(self):
        super(NetworkSocketAddressSchema, self).__init__(
            default_networksocketaddress_properties(), default_networksocketaddress_edges(), lambda: NetworkSocketAddressView
        );

    @staticmethod
    def self_type() -> str:
        return "NetworkSocketAddress"



class NetworkSocketAddressQuery(grapl_analyzerlib.nodes.entity.EntityQuery['NetworkSocketAddressView', 'NetworkSocketAddressQuery']):
    def with_transport_protocol(
        self,
        *,
        eq: Optional[grapl_analyzerlib.comparators.StrOrNot] = None,
        contains: Optional["grapl_analyzerlib.comparators.OneOrMany[grapl_analyzerlib.comparators.StrOrNot]"] = None,
        starts_with: Optional["grapl_analyzerlib.comparators.StrOrNot"] = None,
        ends_with: Optional["grapl_analyzerlib.comparators.StrOrNot"] = None,
        regexp: Optional["grapl_analyzerlib.comparators.OneOrMany[grapl_analyzerlib.comparators.StrOrNot]"] = None,
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
                distance_lt=distance_lt
            )
        )
        return self

    def with_port_number(
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
                "port_number",
                eq=eq,
                gt=gt,
                ge=ge,
                lt=lt,
                le=le,
            )
        )
        return self

    def with_ip_address(
        self,
        *,
        eq: Optional[grapl_analyzerlib.comparators.StrOrNot] = None,
        contains: Optional["grapl_analyzerlib.comparators.OneOrMany[grapl_analyzerlib.comparators.StrOrNot]"] = None,
        starts_with: Optional["grapl_analyzerlib.comparators.StrOrNot"] = None,
        ends_with: Optional["grapl_analyzerlib.comparators.StrOrNot"] = None,
        regexp: Optional["grapl_analyzerlib.comparators.OneOrMany[grapl_analyzerlib.comparators.StrOrNot]"] = None,
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
                distance_lt=distance_lt
            )
        )
        return self

    def with_socket_ipv4_address(self: NetworkSocketAddressQuery, *socket_ipv4_address: IpV4AddressQuery) -> NetworkSocketAddressQuery:
        return self.with_to_neighbor(NetworkSocketAddressQuery, "socket_ipv4_address", "network_sockets_ipv4", socket_ipv4_address)

    def with_socket_ipv6_address(self: NetworkSocketAddressQuery, *socket_ipv6_address: IpV6AddressQuery) -> NetworkSocketAddressQuery:
        return self.with_to_neighbor(NetworkSocketAddressQuery, "socket_ipv6_address", "network_sockets_ipv6", socket_ipv6_address)

    def with_tcp_connection_to_a(self: NetworkSocketAddressQuery, *tcp_connection_to_a: TcpConnectionQuery) -> NetworkSocketAddressQuery:
        return self.with_to_neighbor(NetworkSocketAddressQuery, "tcp_connection_to_a", "tcp_connection_from_b", tcp_connection_to_a)

    def with_tcp_connection_from_a(self: NetworkSocketAddressQuery, *tcp_connection_from_a: TcpConnectionQuery) -> NetworkSocketAddressQuery:
        return self.with_to_neighbor(NetworkSocketAddressQuery, "tcp_connection_from_a", "tcp_connection_to_b", tcp_connection_from_a)

    def with_socket_process_outbound(self: NetworkSocketAddressQuery, *socket_process_outbound: ProcessQuery) -> NetworkSocketAddressQuery:
        return self.with_to_neighbor(NetworkSocketAddressQuery, "socket_process_outbound", "process_socket_outbound", socket_process_outbound)

    def with_socket_process_inbound(self: NetworkSocketAddressQuery, *socket_process_inbound: ProcessQuery) -> NetworkSocketAddressQuery:
        return self.with_to_neighbor(NetworkSocketAddressQuery, "socket_process_inbound", "process_socket_inbound", socket_process_inbound)

    @classmethod
    def node_schema(cls) -> "grapl_analyzerlib.schema.Schema":
        return NetworkSocketAddressSchema()


class NetworkSocketAddressView(grapl_analyzerlib.nodes.entity.EntityView['NetworkSocketAddressView', 'NetworkSocketAddressQuery']):
    queryable = NetworkSocketAddressQuery

    def __init__(
        self,
        uid: int,
        node_key: str,
        graph_client: Any,
        node_types: Set[str],
        transport_protocol: Optional["str"] = None,
        port_number: Optional["int"] = None,
        ip_address: Optional["str"] = None,
        socket_ipv4_address: Optional[List["IpV4AddressView"]] = None,
        socket_ipv6_address: Optional[List["IpV6AddressView"]] = None,
        tcp_connection_to_a: Optional[List["TcpConnectionView"]] = None,
        tcp_connection_from_a: Optional[List["TcpConnectionView"]] = None,
        socket_process_outbound: Optional[List["ProcessView"]] = None,
        socket_process_inbound: Optional[List["ProcessView"]] = None,
        **kwargs,
    ) -> None:
        super().__init__(uid, node_key, graph_client, node_types, **kwargs)
        if transport_protocol: self.set_predicate("transport_protocol", transport_protocol)
        if port_number: self.set_predicate("port_number", port_number)
        if ip_address: self.set_predicate("ip_address", ip_address)
        if socket_ipv4_address: self.set_predicate("socket_ipv4_address", socket_ipv4_address or [])
        if socket_ipv6_address: self.set_predicate("socket_ipv6_address", socket_ipv6_address or [])
        if tcp_connection_to_a: self.set_predicate("tcp_connection_to_a", tcp_connection_to_a or [])
        if tcp_connection_from_a: self.set_predicate("tcp_connection_from_a", tcp_connection_from_a or [])
        if socket_process_outbound: self.set_predicate("socket_process_outbound", socket_process_outbound or [])
        if socket_process_inbound: self.set_predicate("socket_process_inbound", socket_process_inbound or [])

    def get_transport_protocol(self, cached: bool = True) -> Optional[str]:
        return self.get_str("transport_protocol", cached=cached)

    def get_port_number(self, cached: bool = True) -> Optional[int]:
        return self.get_int("port_number", cached=cached)

    def get_ip_address(self, cached: bool = True) -> Optional[str]:
        return self.get_str("ip_address", cached=cached)

    def get_socket_ipv4_address(self, *socket_ipv4_address: IpV4AddressQuery, cached=False) -> 'List[IpV4AddressView]':
          return self.get_neighbor(IpV4AddressQuery, "socket_ipv4_address", "network_sockets_ipv4", socket_ipv4_address, cached)
    def get_socket_ipv6_address(self, *socket_ipv6_address: IpV6AddressQuery, cached=False) -> 'List[IpV6AddressView]':
          return self.get_neighbor(IpV6AddressQuery, "socket_ipv6_address", "network_sockets_ipv6", socket_ipv6_address, cached)
    def get_tcp_connection_to_a(self, *tcp_connection_to_a: TcpConnectionQuery, cached=False) -> 'List[TcpConnectionView]':
          return self.get_neighbor(TcpConnectionQuery, "tcp_connection_to_a", "tcp_connection_from_b", tcp_connection_to_a, cached)
    def get_tcp_connection_from_a(self, *tcp_connection_from_a: TcpConnectionQuery, cached=False) -> 'List[TcpConnectionView]':
          return self.get_neighbor(TcpConnectionQuery, "tcp_connection_from_a", "tcp_connection_to_b", tcp_connection_from_a, cached)
    def get_socket_process_outbound(self, *socket_process_outbound: ProcessQuery, cached=False) -> 'List[ProcessView]':
          return self.get_neighbor(ProcessQuery, "socket_process_outbound", "process_socket_outbound", socket_process_outbound, cached)
    def get_socket_process_inbound(self, *socket_process_inbound: ProcessQuery, cached=False) -> 'List[ProcessView]':
          return self.get_neighbor(ProcessQuery, "socket_process_inbound", "process_socket_inbound", socket_process_inbound, cached)

    @classmethod
    def node_schema(cls) -> "grapl_analyzerlib.schema.Schema":
        return NetworkSocketAddressSchema()

def default_process_properties() -> Dict[str, grapl_analyzerlib.node_types.PropType]:
    return {
        "pid": grapl_analyzerlib.node_types.PropType(grapl_analyzerlib.node_types.PropPrimitive.Int, False),
        "guid": grapl_analyzerlib.node_types.PropType(grapl_analyzerlib.node_types.PropPrimitive.Str, False),
        "exe": grapl_analyzerlib.node_types.PropType(grapl_analyzerlib.node_types.PropPrimitive.Str, False),
    }
def default_process_edges() -> Dict[str, Tuple[grapl_analyzerlib.node_types.EdgeT, str]]:
    return {
        "children": (
            grapl_analyzerlib.node_types.EdgeT(ProcessSchema, ProcessSchema, grapl_analyzerlib.node_types.EdgeRelationship.ManyToMany),
            "parent"
        ),
        "parent": (
            grapl_analyzerlib.node_types.EdgeT(ProcessSchema, ProcessSchema, grapl_analyzerlib.node_types.EdgeRelationship.ManyToMany),
            "children"
        ),
        "spawned_a": (
            grapl_analyzerlib.node_types.EdgeT(ProcessSchema, ProcessSpawnSchema, grapl_analyzerlib.node_types.EdgeRelationship.ManyToMany),
            "spawned_by_a"
        ),
        "spawned_by_b": (
            grapl_analyzerlib.node_types.EdgeT(ProcessSchema, ProcessSpawnSchema, grapl_analyzerlib.node_types.EdgeRelationship.ManyToMany),
            "spawned_b"
        ),
        "process_exe": (
            grapl_analyzerlib.node_types.EdgeT(ProcessSchema, FileSchema, grapl_analyzerlib.node_types.EdgeRelationship.ManyToMany),
            "process_executed_from_exe"
        ),
        "created_files": (
            grapl_analyzerlib.node_types.EdgeT(ProcessSchema, FileSchema, grapl_analyzerlib.node_types.EdgeRelationship.ManyToMany),
            "created_by_process"
        ),
        "process_executed_by_asset": (
            grapl_analyzerlib.node_types.EdgeT(ProcessSchema, AssetSchema, grapl_analyzerlib.node_types.EdgeRelationship.ManyToMany),
            "asset_processes"
        ),
        "process_socket_outbound": (
            grapl_analyzerlib.node_types.EdgeT(ProcessSchema, NetworkSocketAddressSchema, grapl_analyzerlib.node_types.EdgeRelationship.ManyToMany),
            "socket_process_outbound"
        ),
        "process_socket_inbound": (
            grapl_analyzerlib.node_types.EdgeT(ProcessSchema, NetworkSocketAddressSchema, grapl_analyzerlib.node_types.EdgeRelationship.ManyToMany),
            "socket_process_inbound"
        ),
    }

class ProcessSchema(grapl_analyzerlib.nodes.entity.EntitySchema):
    def __init__(self):
        super(ProcessSchema, self).__init__(
            default_process_properties(), default_process_edges(), lambda: ProcessView
        );

    @staticmethod
    def self_type() -> str:
        return "Process"



class ProcessQuery(grapl_analyzerlib.nodes.entity.EntityQuery['ProcessView', 'ProcessQuery']):
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
        contains: Optional["grapl_analyzerlib.comparators.OneOrMany[grapl_analyzerlib.comparators.StrOrNot]"] = None,
        starts_with: Optional["grapl_analyzerlib.comparators.StrOrNot"] = None,
        ends_with: Optional["grapl_analyzerlib.comparators.StrOrNot"] = None,
        regexp: Optional["grapl_analyzerlib.comparators.OneOrMany[grapl_analyzerlib.comparators.StrOrNot]"] = None,
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
                distance_lt=distance_lt
            )
        )
        return self

    def with_exe(
        self,
        *,
        eq: Optional[grapl_analyzerlib.comparators.StrOrNot] = None,
        contains: Optional["grapl_analyzerlib.comparators.OneOrMany[grapl_analyzerlib.comparators.StrOrNot]"] = None,
        starts_with: Optional["grapl_analyzerlib.comparators.StrOrNot"] = None,
        ends_with: Optional["grapl_analyzerlib.comparators.StrOrNot"] = None,
        regexp: Optional["grapl_analyzerlib.comparators.OneOrMany[grapl_analyzerlib.comparators.StrOrNot]"] = None,
        distance_lt: Optional[Tuple[str, int]] = None,
    ):
        (
            self.with_str_property(
                "exe",
                eq=eq,
                contains=contains,
                starts_with=starts_with,
                ends_with=ends_with,
                regexp=regexp,
                distance_lt=distance_lt
            )
        )
        return self

    def with_children(self: ProcessQuery, *children: ProcessQuery) -> ProcessQuery:
        return self.with_to_neighbor(ProcessQuery, "children", "parent", children)

    def with_parent(self: ProcessQuery, *parent: ProcessQuery) -> ProcessQuery:
        return self.with_to_neighbor(ProcessQuery, "parent", "children", parent)

    def with_spawned_a(self: ProcessQuery, *spawned_a: ProcessSpawnQuery) -> ProcessQuery:
        return self.with_to_neighbor(ProcessQuery, "spawned_a", "spawned_by_a", spawned_a)

    def with_spawned_by_b(self: ProcessQuery, *spawned_by_b: ProcessSpawnQuery) -> ProcessQuery:
        return self.with_to_neighbor(ProcessQuery, "spawned_by_b", "spawned_b", spawned_by_b)

    def with_process_exe(self: ProcessQuery, *process_exe: FileQuery) -> ProcessQuery:
        return self.with_to_neighbor(ProcessQuery, "process_exe", "process_executed_from_exe", process_exe)

    def with_created_files(self: ProcessQuery, *created_files: FileQuery) -> ProcessQuery:
        return self.with_to_neighbor(ProcessQuery, "created_files", "created_by_process", created_files)

    def with_process_executed_by_asset(self: ProcessQuery, *process_executed_by_asset: AssetQuery) -> ProcessQuery:
        return self.with_to_neighbor(ProcessQuery, "process_executed_by_asset", "asset_processes", process_executed_by_asset)

    def with_process_socket_outbound(self: ProcessQuery, *process_socket_outbound: NetworkSocketAddressQuery) -> ProcessQuery:
        return self.with_to_neighbor(ProcessQuery, "process_socket_outbound", "socket_process_outbound", process_socket_outbound)

    def with_process_socket_inbound(self: ProcessQuery, *process_socket_inbound: NetworkSocketAddressQuery) -> ProcessQuery:
        return self.with_to_neighbor(ProcessQuery, "process_socket_inbound", "socket_process_inbound", process_socket_inbound)

    @classmethod
    def node_schema(cls) -> "grapl_analyzerlib.schema.Schema":
        return ProcessSchema()


class ProcessView(grapl_analyzerlib.nodes.entity.EntityView['ProcessView', 'ProcessQuery']):
    queryable = ProcessQuery

    def __init__(
        self,
        uid: int,
        node_key: str,
        graph_client: Any,
        node_types: Set[str],
        pid: Optional["int"] = None,
        guid: Optional["str"] = None,
        exe: Optional["str"] = None,
        children: Optional[List["ProcessView"]] = None,
        parent: Optional[List["ProcessView"]] = None,
        spawned_a: Optional[List["ProcessSpawnView"]] = None,
        spawned_by_b: Optional[List["ProcessSpawnView"]] = None,
        process_exe: Optional[List["FileView"]] = None,
        created_files: Optional[List["FileView"]] = None,
        process_executed_by_asset: Optional[List["AssetView"]] = None,
        process_socket_outbound: Optional[List["NetworkSocketAddressView"]] = None,
        process_socket_inbound: Optional[List["NetworkSocketAddressView"]] = None,
        **kwargs,
    ) -> None:
        super().__init__(uid, node_key, graph_client, node_types, **kwargs)
        if pid: self.set_predicate("pid", pid)
        if guid: self.set_predicate("guid", guid)
        if exe: self.set_predicate("exe", exe)
        if children: self.set_predicate("children", children or [])
        if parent: self.set_predicate("parent", parent or [])
        if spawned_a: self.set_predicate("spawned_a", spawned_a or [])
        if spawned_by_b: self.set_predicate("spawned_by_b", spawned_by_b or [])
        if process_exe: self.set_predicate("process_exe", process_exe or [])
        if created_files: self.set_predicate("created_files", created_files or [])
        if process_executed_by_asset: self.set_predicate("process_executed_by_asset", process_executed_by_asset or [])
        if process_socket_outbound: self.set_predicate("process_socket_outbound", process_socket_outbound or [])
        if process_socket_inbound: self.set_predicate("process_socket_inbound", process_socket_inbound or [])

    def get_pid(self, cached: bool = True) -> Optional[int]:
        return self.get_int("pid", cached=cached)

    def get_guid(self, cached: bool = True) -> Optional[str]:
        return self.get_str("guid", cached=cached)

    def get_exe(self, cached: bool = True) -> Optional[str]:
        return self.get_str("exe", cached=cached)

    def get_children(self, *children: ProcessQuery, cached=False) -> 'List[ProcessView]':
          return self.get_neighbor(ProcessQuery, "children", "parent", children, cached)
    def get_parent(self, *parent: ProcessQuery, cached=False) -> 'List[ProcessView]':
          return self.get_neighbor(ProcessQuery, "parent", "children", parent, cached)
    def get_spawned_a(self, *spawned_a: ProcessSpawnQuery, cached=False) -> 'List[ProcessSpawnView]':
          return self.get_neighbor(ProcessSpawnQuery, "spawned_a", "spawned_by_a", spawned_a, cached)
    def get_spawned_by_b(self, *spawned_by_b: ProcessSpawnQuery, cached=False) -> 'List[ProcessSpawnView]':
          return self.get_neighbor(ProcessSpawnQuery, "spawned_by_b", "spawned_b", spawned_by_b, cached)
    def get_process_exe(self, *process_exe: FileQuery, cached=False) -> 'List[FileView]':
          return self.get_neighbor(FileQuery, "process_exe", "process_executed_from_exe", process_exe, cached)
    def get_created_files(self, *created_files: FileQuery, cached=False) -> 'List[FileView]':
          return self.get_neighbor(FileQuery, "created_files", "created_by_process", created_files, cached)
    def get_process_executed_by_asset(self, *process_executed_by_asset: AssetQuery, cached=False) -> 'List[AssetView]':
          return self.get_neighbor(AssetQuery, "process_executed_by_asset", "asset_processes", process_executed_by_asset, cached)
    def get_process_socket_outbound(self, *process_socket_outbound: NetworkSocketAddressQuery, cached=False) -> 'List[NetworkSocketAddressView]':
          return self.get_neighbor(NetworkSocketAddressQuery, "process_socket_outbound", "socket_process_outbound", process_socket_outbound, cached)
    def get_process_socket_inbound(self, *process_socket_inbound: NetworkSocketAddressQuery, cached=False) -> 'List[NetworkSocketAddressView]':
          return self.get_neighbor(NetworkSocketAddressQuery, "process_socket_inbound", "socket_process_inbound", process_socket_inbound, cached)

    @classmethod
    def node_schema(cls) -> "grapl_analyzerlib.schema.Schema":
        return ProcessSchema()

def default_processspawn_properties() -> Dict[str, grapl_analyzerlib.node_types.PropType]:
    return {
        "timestamp": grapl_analyzerlib.node_types.PropType(grapl_analyzerlib.node_types.PropPrimitive.Int, False),
        "user_id": grapl_analyzerlib.node_types.PropType(grapl_analyzerlib.node_types.PropPrimitive.Int, False),
        "user": grapl_analyzerlib.node_types.PropType(grapl_analyzerlib.node_types.PropPrimitive.Str, False),
        "parent_user": grapl_analyzerlib.node_types.PropType(grapl_analyzerlib.node_types.PropPrimitive.Str, False),
        "cmdline": grapl_analyzerlib.node_types.PropType(grapl_analyzerlib.node_types.PropPrimitive.Str, False),
        "current_directory": grapl_analyzerlib.node_types.PropType(grapl_analyzerlib.node_types.PropPrimitive.Str, False),
        "parent_guid": grapl_analyzerlib.node_types.PropType(grapl_analyzerlib.node_types.PropPrimitive.Str, False),
        "child_guid": grapl_analyzerlib.node_types.PropType(grapl_analyzerlib.node_types.PropPrimitive.Str, False),
    }
def default_processspawn_edges() -> Dict[str, Tuple[grapl_analyzerlib.node_types.EdgeT, str]]:
    return {
        "spawned_b": (
            grapl_analyzerlib.node_types.EdgeT(ProcessSpawnSchema, ProcessSchema, grapl_analyzerlib.node_types.EdgeRelationship.ManyToMany),
            "spawned_by_b"
        ),
        "spawned_by_a": (
            grapl_analyzerlib.node_types.EdgeT(ProcessSpawnSchema, ProcessSchema, grapl_analyzerlib.node_types.EdgeRelationship.ManyToMany),
            "spawned_a"
        ),
    }

class ProcessSpawnSchema(grapl_analyzerlib.nodes.entity.EntitySchema):
    def __init__(self):
        super(ProcessSpawnSchema, self).__init__(
            default_processspawn_properties(), default_processspawn_edges(), lambda: ProcessSpawnView
        );

    @staticmethod
    def self_type() -> str:
        return "ProcessSpawn"



class ProcessSpawnQuery(grapl_analyzerlib.nodes.entity.EntityQuery['ProcessSpawnView', 'ProcessSpawnQuery']):
    def with_timestamp(
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
                "timestamp",
                eq=eq,
                gt=gt,
                ge=ge,
                lt=lt,
                le=le,
            )
        )
        return self

    def with_user_id(
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
                "user_id",
                eq=eq,
                gt=gt,
                ge=ge,
                lt=lt,
                le=le,
            )
        )
        return self

    def with_user(
        self,
        *,
        eq: Optional[grapl_analyzerlib.comparators.StrOrNot] = None,
        contains: Optional["grapl_analyzerlib.comparators.OneOrMany[grapl_analyzerlib.comparators.StrOrNot]"] = None,
        starts_with: Optional["grapl_analyzerlib.comparators.StrOrNot"] = None,
        ends_with: Optional["grapl_analyzerlib.comparators.StrOrNot"] = None,
        regexp: Optional["grapl_analyzerlib.comparators.OneOrMany[grapl_analyzerlib.comparators.StrOrNot]"] = None,
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
                distance_lt=distance_lt
            )
        )
        return self

    def with_parent_user(
        self,
        *,
        eq: Optional[grapl_analyzerlib.comparators.StrOrNot] = None,
        contains: Optional["grapl_analyzerlib.comparators.OneOrMany[grapl_analyzerlib.comparators.StrOrNot]"] = None,
        starts_with: Optional["grapl_analyzerlib.comparators.StrOrNot"] = None,
        ends_with: Optional["grapl_analyzerlib.comparators.StrOrNot"] = None,
        regexp: Optional["grapl_analyzerlib.comparators.OneOrMany[grapl_analyzerlib.comparators.StrOrNot]"] = None,
        distance_lt: Optional[Tuple[str, int]] = None,
    ):
        (
            self.with_str_property(
                "parent_user",
                eq=eq,
                contains=contains,
                starts_with=starts_with,
                ends_with=ends_with,
                regexp=regexp,
                distance_lt=distance_lt
            )
        )
        return self

    def with_cmdline(
        self,
        *,
        eq: Optional[grapl_analyzerlib.comparators.StrOrNot] = None,
        contains: Optional["grapl_analyzerlib.comparators.OneOrMany[grapl_analyzerlib.comparators.StrOrNot]"] = None,
        starts_with: Optional["grapl_analyzerlib.comparators.StrOrNot"] = None,
        ends_with: Optional["grapl_analyzerlib.comparators.StrOrNot"] = None,
        regexp: Optional["grapl_analyzerlib.comparators.OneOrMany[grapl_analyzerlib.comparators.StrOrNot]"] = None,
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
                distance_lt=distance_lt
            )
        )
        return self

    def with_current_directory(
        self,
        *,
        eq: Optional[grapl_analyzerlib.comparators.StrOrNot] = None,
        contains: Optional["grapl_analyzerlib.comparators.OneOrMany[grapl_analyzerlib.comparators.StrOrNot]"] = None,
        starts_with: Optional["grapl_analyzerlib.comparators.StrOrNot"] = None,
        ends_with: Optional["grapl_analyzerlib.comparators.StrOrNot"] = None,
        regexp: Optional["grapl_analyzerlib.comparators.OneOrMany[grapl_analyzerlib.comparators.StrOrNot]"] = None,
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
                distance_lt=distance_lt
            )
        )
        return self

    def with_parent_guid(
        self,
        *,
        eq: Optional[grapl_analyzerlib.comparators.StrOrNot] = None,
        contains: Optional["grapl_analyzerlib.comparators.OneOrMany[grapl_analyzerlib.comparators.StrOrNot]"] = None,
        starts_with: Optional["grapl_analyzerlib.comparators.StrOrNot"] = None,
        ends_with: Optional["grapl_analyzerlib.comparators.StrOrNot"] = None,
        regexp: Optional["grapl_analyzerlib.comparators.OneOrMany[grapl_analyzerlib.comparators.StrOrNot]"] = None,
        distance_lt: Optional[Tuple[str, int]] = None,
    ):
        (
            self.with_str_property(
                "parent_guid",
                eq=eq,
                contains=contains,
                starts_with=starts_with,
                ends_with=ends_with,
                regexp=regexp,
                distance_lt=distance_lt
            )
        )
        return self

    def with_child_guid(
        self,
        *,
        eq: Optional[grapl_analyzerlib.comparators.StrOrNot] = None,
        contains: Optional["grapl_analyzerlib.comparators.OneOrMany[grapl_analyzerlib.comparators.StrOrNot]"] = None,
        starts_with: Optional["grapl_analyzerlib.comparators.StrOrNot"] = None,
        ends_with: Optional["grapl_analyzerlib.comparators.StrOrNot"] = None,
        regexp: Optional["grapl_analyzerlib.comparators.OneOrMany[grapl_analyzerlib.comparators.StrOrNot]"] = None,
        distance_lt: Optional[Tuple[str, int]] = None,
    ):
        (
            self.with_str_property(
                "child_guid",
                eq=eq,
                contains=contains,
                starts_with=starts_with,
                ends_with=ends_with,
                regexp=regexp,
                distance_lt=distance_lt
            )
        )
        return self

    def with_spawned_b(self: ProcessSpawnQuery, *spawned_b: ProcessQuery) -> ProcessSpawnQuery:
        return self.with_to_neighbor(ProcessSpawnQuery, "spawned_b", "spawned_by_b", spawned_b)

    def with_spawned_by_a(self: ProcessSpawnQuery, *spawned_by_a: ProcessQuery) -> ProcessSpawnQuery:
        return self.with_to_neighbor(ProcessSpawnQuery, "spawned_by_a", "spawned_a", spawned_by_a)

    @classmethod
    def node_schema(cls) -> "grapl_analyzerlib.schema.Schema":
        return ProcessSpawnSchema()


class ProcessSpawnView(grapl_analyzerlib.nodes.entity.EntityView['ProcessSpawnView', 'ProcessSpawnQuery']):
    queryable = ProcessSpawnQuery

    def __init__(
        self,
        uid: int,
        node_key: str,
        graph_client: Any,
        node_types: Set[str],
        timestamp: Optional["int"] = None,
        user_id: Optional["int"] = None,
        user: Optional["str"] = None,
        parent_user: Optional["str"] = None,
        cmdline: Optional["str"] = None,
        current_directory: Optional["str"] = None,
        parent_guid: Optional["str"] = None,
        child_guid: Optional["str"] = None,
        spawned_b: Optional[List["ProcessView"]] = None,
        spawned_by_a: Optional[List["ProcessView"]] = None,
        **kwargs,
    ) -> None:
        super().__init__(uid, node_key, graph_client, node_types, **kwargs)
        if timestamp: self.set_predicate("timestamp", timestamp)
        if user_id: self.set_predicate("user_id", user_id)
        if user: self.set_predicate("user", user)
        if parent_user: self.set_predicate("parent_user", parent_user)
        if cmdline: self.set_predicate("cmdline", cmdline)
        if current_directory: self.set_predicate("current_directory", current_directory)
        if parent_guid: self.set_predicate("parent_guid", parent_guid)
        if child_guid: self.set_predicate("child_guid", child_guid)
        if spawned_b: self.set_predicate("spawned_b", spawned_b or [])
        if spawned_by_a: self.set_predicate("spawned_by_a", spawned_by_a or [])

    def get_timestamp(self, cached: bool = True) -> Optional[int]:
        return self.get_int("timestamp", cached=cached)

    def get_user_id(self, cached: bool = True) -> Optional[int]:
        return self.get_int("user_id", cached=cached)

    def get_user(self, cached: bool = True) -> Optional[str]:
        return self.get_str("user", cached=cached)

    def get_parent_user(self, cached: bool = True) -> Optional[str]:
        return self.get_str("parent_user", cached=cached)

    def get_cmdline(self, cached: bool = True) -> Optional[str]:
        return self.get_str("cmdline", cached=cached)

    def get_current_directory(self, cached: bool = True) -> Optional[str]:
        return self.get_str("current_directory", cached=cached)

    def get_parent_guid(self, cached: bool = True) -> Optional[str]:
        return self.get_str("parent_guid", cached=cached)

    def get_child_guid(self, cached: bool = True) -> Optional[str]:
        return self.get_str("child_guid", cached=cached)

    def get_spawned_b(self, *spawned_b: ProcessQuery, cached=False) -> 'List[ProcessView]':
          return self.get_neighbor(ProcessQuery, "spawned_b", "spawned_by_b", spawned_b, cached)
    def get_spawned_by_a(self, *spawned_by_a: ProcessQuery, cached=False) -> 'List[ProcessView]':
          return self.get_neighbor(ProcessQuery, "spawned_by_a", "spawned_a", spawned_by_a, cached)

    @classmethod
    def node_schema(cls) -> "grapl_analyzerlib.schema.Schema":
        return ProcessSpawnSchema()

def default_tcpconnection_properties() -> Dict[str, grapl_analyzerlib.node_types.PropType]:
    return {
        "created_timestamp": grapl_analyzerlib.node_types.PropType(grapl_analyzerlib.node_types.PropPrimitive.Int, False),
        "src_port": grapl_analyzerlib.node_types.PropType(grapl_analyzerlib.node_types.PropPrimitive.Int, False),
        "dst_port": grapl_analyzerlib.node_types.PropType(grapl_analyzerlib.node_types.PropPrimitive.Int, False),
        "src_ip_address": grapl_analyzerlib.node_types.PropType(grapl_analyzerlib.node_types.PropPrimitive.Str, False),
        "dst_ip_address": grapl_analyzerlib.node_types.PropType(grapl_analyzerlib.node_types.PropPrimitive.Str, False),
        "transport_protocol": grapl_analyzerlib.node_types.PropType(grapl_analyzerlib.node_types.PropPrimitive.Str, False),
        "process_guid": grapl_analyzerlib.node_types.PropType(grapl_analyzerlib.node_types.PropPrimitive.Str, False),
    }
def default_tcpconnection_edges() -> Dict[str, Tuple[grapl_analyzerlib.node_types.EdgeT, str]]:
    return {
        "tcp_connection_to_b": (
            grapl_analyzerlib.node_types.EdgeT(TcpConnectionSchema, NetworkSocketAddressSchema, grapl_analyzerlib.node_types.EdgeRelationship.ManyToMany),
            "tcp_connection_from_a"
        ),
        "tcp_connection_from_b": (
            grapl_analyzerlib.node_types.EdgeT(TcpConnectionSchema, NetworkSocketAddressSchema, grapl_analyzerlib.node_types.EdgeRelationship.ManyToMany),
            "tcp_connection_to_a"
        ),
    }

class TcpConnectionSchema(grapl_analyzerlib.nodes.entity.EntitySchema):
    def __init__(self):
        super(TcpConnectionSchema, self).__init__(
            default_tcpconnection_properties(), default_tcpconnection_edges(), lambda: TcpConnectionView
        );

    @staticmethod
    def self_type() -> str:
        return "TcpConnection"



class TcpConnectionQuery(grapl_analyzerlib.nodes.entity.EntityQuery['TcpConnectionView', 'TcpConnectionQuery']):
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

    def with_src_port(
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
                "src_port",
                eq=eq,
                gt=gt,
                ge=ge,
                lt=lt,
                le=le,
            )
        )
        return self

    def with_dst_port(
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
                "dst_port",
                eq=eq,
                gt=gt,
                ge=ge,
                lt=lt,
                le=le,
            )
        )
        return self

    def with_src_ip_address(
        self,
        *,
        eq: Optional[grapl_analyzerlib.comparators.StrOrNot] = None,
        contains: Optional["grapl_analyzerlib.comparators.OneOrMany[grapl_analyzerlib.comparators.StrOrNot]"] = None,
        starts_with: Optional["grapl_analyzerlib.comparators.StrOrNot"] = None,
        ends_with: Optional["grapl_analyzerlib.comparators.StrOrNot"] = None,
        regexp: Optional["grapl_analyzerlib.comparators.OneOrMany[grapl_analyzerlib.comparators.StrOrNot]"] = None,
        distance_lt: Optional[Tuple[str, int]] = None,
    ):
        (
            self.with_str_property(
                "src_ip_address",
                eq=eq,
                contains=contains,
                starts_with=starts_with,
                ends_with=ends_with,
                regexp=regexp,
                distance_lt=distance_lt
            )
        )
        return self

    def with_dst_ip_address(
        self,
        *,
        eq: Optional[grapl_analyzerlib.comparators.StrOrNot] = None,
        contains: Optional["grapl_analyzerlib.comparators.OneOrMany[grapl_analyzerlib.comparators.StrOrNot]"] = None,
        starts_with: Optional["grapl_analyzerlib.comparators.StrOrNot"] = None,
        ends_with: Optional["grapl_analyzerlib.comparators.StrOrNot"] = None,
        regexp: Optional["grapl_analyzerlib.comparators.OneOrMany[grapl_analyzerlib.comparators.StrOrNot]"] = None,
        distance_lt: Optional[Tuple[str, int]] = None,
    ):
        (
            self.with_str_property(
                "dst_ip_address",
                eq=eq,
                contains=contains,
                starts_with=starts_with,
                ends_with=ends_with,
                regexp=regexp,
                distance_lt=distance_lt
            )
        )
        return self

    def with_transport_protocol(
        self,
        *,
        eq: Optional[grapl_analyzerlib.comparators.StrOrNot] = None,
        contains: Optional["grapl_analyzerlib.comparators.OneOrMany[grapl_analyzerlib.comparators.StrOrNot]"] = None,
        starts_with: Optional["grapl_analyzerlib.comparators.StrOrNot"] = None,
        ends_with: Optional["grapl_analyzerlib.comparators.StrOrNot"] = None,
        regexp: Optional["grapl_analyzerlib.comparators.OneOrMany[grapl_analyzerlib.comparators.StrOrNot]"] = None,
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
                distance_lt=distance_lt
            )
        )
        return self

    def with_process_guid(
        self,
        *,
        eq: Optional[grapl_analyzerlib.comparators.StrOrNot] = None,
        contains: Optional["grapl_analyzerlib.comparators.OneOrMany[grapl_analyzerlib.comparators.StrOrNot]"] = None,
        starts_with: Optional["grapl_analyzerlib.comparators.StrOrNot"] = None,
        ends_with: Optional["grapl_analyzerlib.comparators.StrOrNot"] = None,
        regexp: Optional["grapl_analyzerlib.comparators.OneOrMany[grapl_analyzerlib.comparators.StrOrNot]"] = None,
        distance_lt: Optional[Tuple[str, int]] = None,
    ):
        (
            self.with_str_property(
                "process_guid",
                eq=eq,
                contains=contains,
                starts_with=starts_with,
                ends_with=ends_with,
                regexp=regexp,
                distance_lt=distance_lt
            )
        )
        return self

    def with_tcp_connection_to_b(self: TcpConnectionQuery, *tcp_connection_to_b: NetworkSocketAddressQuery) -> TcpConnectionQuery:
        return self.with_to_neighbor(TcpConnectionQuery, "tcp_connection_to_b", "tcp_connection_from_a", tcp_connection_to_b)

    def with_tcp_connection_from_b(self: TcpConnectionQuery, *tcp_connection_from_b: NetworkSocketAddressQuery) -> TcpConnectionQuery:
        return self.with_to_neighbor(TcpConnectionQuery, "tcp_connection_from_b", "tcp_connection_to_a", tcp_connection_from_b)

    @classmethod
    def node_schema(cls) -> "grapl_analyzerlib.schema.Schema":
        return TcpConnectionSchema()


class TcpConnectionView(grapl_analyzerlib.nodes.entity.EntityView['TcpConnectionView', 'TcpConnectionQuery']):
    queryable = TcpConnectionQuery

    def __init__(
        self,
        uid: int,
        node_key: str,
        graph_client: Any,
        node_types: Set[str],
        created_timestamp: Optional["int"] = None,
        src_port: Optional["int"] = None,
        dst_port: Optional["int"] = None,
        src_ip_address: Optional["str"] = None,
        dst_ip_address: Optional["str"] = None,
        transport_protocol: Optional["str"] = None,
        process_guid: Optional["str"] = None,
        tcp_connection_to_b: Optional[List["NetworkSocketAddressView"]] = None,
        tcp_connection_from_b: Optional[List["NetworkSocketAddressView"]] = None,
        **kwargs,
    ) -> None:
        super().__init__(uid, node_key, graph_client, node_types, **kwargs)
        if created_timestamp: self.set_predicate("created_timestamp", created_timestamp)
        if src_port: self.set_predicate("src_port", src_port)
        if dst_port: self.set_predicate("dst_port", dst_port)
        if src_ip_address: self.set_predicate("src_ip_address", src_ip_address)
        if dst_ip_address: self.set_predicate("dst_ip_address", dst_ip_address)
        if transport_protocol: self.set_predicate("transport_protocol", transport_protocol)
        if process_guid: self.set_predicate("process_guid", process_guid)
        if tcp_connection_to_b: self.set_predicate("tcp_connection_to_b", tcp_connection_to_b or [])
        if tcp_connection_from_b: self.set_predicate("tcp_connection_from_b", tcp_connection_from_b or [])

    def get_created_timestamp(self, cached: bool = True) -> Optional[int]:
        return self.get_int("created_timestamp", cached=cached)

    def get_src_port(self, cached: bool = True) -> Optional[int]:
        return self.get_int("src_port", cached=cached)

    def get_dst_port(self, cached: bool = True) -> Optional[int]:
        return self.get_int("dst_port", cached=cached)

    def get_src_ip_address(self, cached: bool = True) -> Optional[str]:
        return self.get_str("src_ip_address", cached=cached)

    def get_dst_ip_address(self, cached: bool = True) -> Optional[str]:
        return self.get_str("dst_ip_address", cached=cached)

    def get_transport_protocol(self, cached: bool = True) -> Optional[str]:
        return self.get_str("transport_protocol", cached=cached)

    def get_process_guid(self, cached: bool = True) -> Optional[str]:
        return self.get_str("process_guid", cached=cached)

    def get_tcp_connection_to_b(self, *tcp_connection_to_b: NetworkSocketAddressQuery, cached=False) -> 'List[NetworkSocketAddressView]':
          return self.get_neighbor(NetworkSocketAddressQuery, "tcp_connection_to_b", "tcp_connection_from_a", tcp_connection_to_b, cached)
    def get_tcp_connection_from_b(self, *tcp_connection_from_b: NetworkSocketAddressQuery, cached=False) -> 'List[NetworkSocketAddressView]':
          return self.get_neighbor(NetworkSocketAddressQuery, "tcp_connection_from_b", "tcp_connection_to_a", tcp_connection_from_b, cached)

    @classmethod
    def node_schema(cls) -> "grapl_analyzerlib.schema.Schema":
        return TcpConnectionSchema()


