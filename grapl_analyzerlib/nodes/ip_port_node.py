from typing import *

from pydgraph import DgraphClient

from grapl_analyzerlib.nodes.comparators import (
    Cmp,
    IntCmp,
    _int_cmps,
    StrCmp,
    _str_cmps,
    PropertyFilter,
)
from grapl_analyzerlib.nodes.queryable import Queryable, NQ
from grapl_analyzerlib.nodes.types import PropertyT, Property
from grapl_analyzerlib.nodes.viewable import (
    EdgeViewT,
    ForwardEdgeView,
    Viewable,
    ReverseEdgeView,
)

IIpPortQuery = TypeVar("IIpPortQuery", bound="IpPortQuery")


class IpPortQuery(Queryable):
    def __init__(self):
        super(IpPortQuery, self).__init__(IpPortView)
        self._port = []  # type: List[List[Cmp[int]]]
        self._first_seen_timestamp = []  # type: List[List[Cmp[int]]]
        self._last_seen_timestamp = []  # type: List[List[Cmp[int]]]

        self._ip_address = []  # type: List[List[Cmp[str]]]
        self._protocol = []  # type: List[List[Cmp[str]]]

        self._network_connections = None  # type: Optional[INetworkConnectionQuery]

    def with_ip_address(
        self,
        eq: Optional["StrCmp"] = None,
        contains: Optional["StrCmp"] = None,
        ends_with: Optional["StrCmp"] = None,
        starts_with: Optional["StrCmp"] = None,
    ) -> "NQ":
        self.set_str_property_filter(
            "ip_address",
            _str_cmps("ip_address", eq=eq, contains=contains, ends_with=ends_with, starts_with=starts_with),
        )
        return self

    def with_protocol(
        self,
        eq: Optional["StrCmp"] = None,
        contains: Optional["StrCmp"] = None,
        ends_with: Optional["StrCmp"] = None,
    ) -> "NQ":
        self.set_str_property_filter(
            "protocol",
            _str_cmps("protocol", eq=eq, contains=contains, ends_with=ends_with),
        )
        return self

    def with_port(
        self: "NQ",
        eq: Optional["IntCmp"] = None,
        gt: Optional["IntCmp"] = None,
        lt: Optional["IntCmp"] = None,
    ) -> "NQ":
        self.set_int_property_filter("port", _int_cmps("port", eq=eq, gt=gt, lt=lt))
        return self

    def with_first_seen_timestamp(
        self: "NQ",
        eq: Optional["IntCmp"] = None,
        gt: Optional["IntCmp"] = None,
        lt: Optional["IntCmp"] = None,
    ) -> "NQ":
        self.set_int_property_filter(
            "first_seen_timestamp",
            _int_cmps("first_seen_timestamp", eq=eq, gt=gt, lt=lt),
        )
        return self

    def with_last_seen_timestamp(
        self: "NQ",
        eq: Optional["IntCmp"] = None,
        gt: Optional["IntCmp"] = None,
        lt: Optional["IntCmp"] = None,
    ) -> "NQ":
        self.set_int_property_filter(
            "last_seen_timestamp", _int_cmps("last_seen_timestamp", eq=eq, gt=gt, lt=lt)
        )
        return self

    def with_network_connections(
        self: "NQ",
        network_connections_query: Optional["INetworkConnectionQuery"] = None,
    ) -> "NQ":
        network_connections = network_connections_query or NetworkConnectionQuery()

        self.set_forward_edge_filter("network_connections", network_connections)
        network_connections.set_reverse_edge_filter(
            "~network_connections", self, "network_connections"
        )
        return self

    def with_network_connections_from(
        self: "NQ",
        network_connections_from_query: Optional["INetworkConnectionQuery"] = None,
    ) -> "NQ":
        network_connections_from = (
            network_connections_from_query or NetworkConnectionQuery()
        )
        network_connections_from.with_inbound_connection_to(cast(IpPortQuery, self))

        return self

    def with_bound_by(
        self: "NQ",
        bound_by_query: Optional["IProcessInboundConnectionQuery"] = None,
    ) -> "NQ":
        bound_by = bound_by_query or ProcessInboundConnectionQuery()
        bound_by.with_bound_port(cast(IpPortQuery, self))

        return self

    def with_process_connections(
        self: "NQ",
        process_connections_query: Optional[
            "IProcessOutboundConnectionQuery"
        ] = None,
    ) -> "NQ":
        process_connections = (
            process_connections_query or ProcessOutboundConnectionQuery()
        )
        process_connections.with_connected_over(cast(IpPortQuery, self))

        return self

    def with_connections_from_processes(
        self: "NQ",
        connections_from_processes_query: Optional[
            "IProcessOutboundConnectionQuery"
        ] = None,
    ) -> "NQ":
        connections_from_processes = (
            connections_from_processes_query or ProcessOutboundConnectionQuery()
        )
        connections_from_processes.with_process_outbound_connection(
            cast(IpPortQuery, self)
        )

        return self

    def _get_unique_predicate(self) -> Optional[Tuple[str, "PropertyT"]]:
        return None

    def _get_node_type_name(self) -> str:
        return "IpPort"

    def _get_property_filters(self) -> Mapping[str, "PropertyFilter[Property]"]:
        props = {
            "port": self._port,
            "first_seen_timestamp": self._first_seen_timestamp,
            "last_seen_timestamp": self._last_seen_timestamp,
            "ip_address": self._ip_address,
            "protocol": self._protocol,
        }

        combined = {}
        for prop_name, prop_filter in props.items():
            if prop_filter:
                combined[prop_name] = cast("PropertyFilter[Property]", prop_filter)

        return combined

    def _get_forward_edges(self) -> Mapping[str, "Queryable"]:
        forward_edges = {"network_connections": self._network_connections}

        return {fe[0]: fe[1] for fe in forward_edges.items() if fe[1] is not None}

    def _get_reverse_edges(self) -> Mapping[str, Tuple["Queryable", str]]:
        return {}


IIpPortView = TypeVar("IIpPortView", bound="IpPortView")


class IpPortView(Viewable):
    def __init__(
        self,
        dgraph_client: DgraphClient,
        node_key: str,
        uid: str,
        node_type: str,
        port: Optional[int] = None,
        first_seen_timestamp: Optional[int] = None,
        last_seen_timestamp: Optional[int] = None,
        ip_address: Optional[str] = None,
        protocol: Optional[str] = None,
        network_connections: "Optional[List[NetworkConnectionView]]" = None,
        bound_by: "Optional[List[ProcessInboundConnectionView]]" = None,
        process_connections: "Optional[List[ProcessOutboundConnectionView]]" = None,
        process_connects: "Optional[List[ProcessOutboundConnectionView]]" = None,
    ):
        super(IpPortView, self).__init__(
            dgraph_client=dgraph_client, node_key=node_key, uid=uid, node_type=node_type
        )
        self.dgraph_client = dgraph_client
        self.node_key = node_key
        self.uid = uid
        self.node_type = node_type

        self.port = port
        self.first_seen_timestamp = first_seen_timestamp
        self.last_seen_timestamp = last_seen_timestamp
        self.ip_address = ip_address
        self.protocol = protocol

        # Forward edges
        self.network_connections = network_connections

        # Reverse edges
        # Processes that have bound this iP + Port
        self.bound_by = bound_by
        # Connections created by processes
        self.process_connections = process_connections
        # Process connects overt his port
        self.process_connects = process_connects

    def get_node_type(self) -> str:
        return 'IpPort'

    def get_port(self) -> Optional[int]:
        if not self.port:
            self.port = cast(Optional[int], self.fetch_property("port", int))
        return self.port

    def get_first_seen_timestamp(self) -> Optional[int]:
        if not self.first_seen_timestamp:
            self.first_seen_timestamp = cast(
                Optional[int], self.fetch_property("first_seen_timestamp", int)
            )
        return self.first_seen_timestamp

    def get_last_seen_timestamp(self) -> Optional[int]:
        if not self.last_seen_timestamp:
            self.last_seen_timestamp = cast(
                Optional[int], self.fetch_property("last_seen_timestamp", int)
            )
        return self.last_seen_timestamp

    def get_ip_address(self) -> Optional[str]:
        if not self.ip_address:
            self.ip_address = cast(
                Optional[str], self.fetch_property("ip_address", str)
            )
        return self.ip_address

    def get_protocol(self) -> Optional[str]:
        if not self.protocol:
            self.protocol = cast(Optional[str], self.fetch_property("protocol", str))
        return self.protocol

    def get_network_connections_from(self,) -> "List[NetworkConnectionView]":
        return cast(
            List[NetworkConnectionView],
            self.fetch_edges("~inbound_connection_to", NetworkConnectionView),
        )

    def get_bound_by(self,) -> "List[ProcessInboundConnectionView]":
        return cast(
            List[ProcessInboundConnectionView],
            self.fetch_edges("~bound_port", ProcessInboundConnectionView),
        )

    def get_process_connects(self,) -> "List[ProcessOutboundConnectionView]":
        return cast(
            List[ProcessOutboundConnectionView],
            self.fetch_edges("~connected_over", ProcessOutboundConnectionView),
        )

    def get_connections_from_processes(
        self,
    ) -> "List[ProcessOutboundConnectionView]":
        return cast(
            List[ProcessOutboundConnectionView],
            self.fetch_edges(
                "~process_outbound_connection", ProcessOutboundConnectionView
            ),
        )

    @staticmethod
    def _get_property_types() -> Mapping[str, "PropertyT"]:
        return {
            "port": int,
            "first_seen_timestamp": int,
            "last_seen_timestamp": int,
            "ip_address": str,
            "protocol": str,
        }

    def _get_properties(self, fetch: bool = False) -> Mapping[str, Union[str, int]]:
        props = {
            "port": self.port,
            "first_seen_timestamp": self.first_seen_timestamp,
            "last_seen_timestamp": self.last_seen_timestamp,
            "ip_address": self.ip_address,
            "protocol": self.protocol,
        }

        return {p[0]: p[1] for p in props.items() if p[1] is not None}

    @staticmethod
    def _get_forward_edge_types() -> Mapping[str, "EdgeViewT"]:
        f_edges = {
            "network_connections": [NetworkConnectionView]
        }  # type: Dict[str, Optional["EdgeViewT"]]

        return cast(
            Mapping[str, "EdgeViewT"], {fe[0]: fe[1] for fe in f_edges.items() if fe[1]}
        )

    def _get_forward_edges(self) -> "Mapping[str, ForwardEdgeView]":
        f_edges = {
            "network_connections": self.network_connections
        }  # type: Dict[str, Optional[ForwardEdgeView]]

        return cast(
            Mapping[str, ForwardEdgeView],
            {fe[0]: fe[1] for fe in f_edges.items() if fe[1]},
        )

    @staticmethod
    def _get_reverse_edge_types() -> Mapping[str, Tuple["EdgeViewT", str]]:
        return {
            "~bound_port": ([ProcessInboundConnectionView], "bound_by"),
            "~connected_over": ([ProcessInboundConnectionView], "bound_by"),
        }

    def _get_reverse_edges(self) -> Mapping[str, Tuple["Queryable", str]]:
        reverse_edges = {
            "~created_connections": (self.process_connections, "connecting_processes")
        }

        return {
            fe[0]: (fe[1][0], fe[1][1])
            for fe in reverse_edges.items()
            if fe[1][0] is not None
        }


from grapl_analyzerlib.nodes.network_connection_node import (
    NetworkConnectionView,
    INetworkConnectionQuery,
    NetworkConnectionQuery,
)

from grapl_analyzerlib.nodes.process_inbound_network_connection import (
    ProcessInboundConnectionQuery,
    IProcessInboundConnectionQuery,
    ProcessInboundConnectionView,
)

from grapl_analyzerlib.nodes.process_outbound_network_connection import (
    IProcessOutboundConnectionQuery,
    ProcessOutboundConnectionQuery,
    ProcessOutboundConnectionView,
)
