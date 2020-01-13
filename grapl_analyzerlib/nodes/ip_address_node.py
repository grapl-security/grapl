from typing import *

from pydgraph import DgraphClient

from grapl_analyzerlib.nodes.comparators import (
    Cmp,
    IntCmp,
    _int_cmps,
    StrCmp,
    _str_cmps,
)
from grapl_analyzerlib.nodes.dynamic_node import DynamicNodeQuery, DynamicNodeView
from grapl_analyzerlib.nodes.queryable import NQ
from grapl_analyzerlib.nodes.types import PropertyT
from grapl_analyzerlib.nodes.viewable import EdgeViewT, ForwardEdgeView

IIpAddressQuery = TypeVar("IIpAddressQuery", bound="IpAddressQuery")


class IpAddressQuery(DynamicNodeQuery):
    def __init__(self):
        super(IpAddressQuery, self).__init__("IpAddress", IpAddressView)
        self._first_seen_timestamp = []  # type: List[List[Cmp[int]]]
        self._last_seen_timestamp = []  # type: List[List[Cmp[int]]]

        self._ip_address = []  # type: List[List[Cmp[str]]]

        self._ip_connections = None  # type: Optional[IIpConnectionQuery]

    def with_ip_address(
        self,
        eq: Optional["StrCmp"] = None,
        contains: Optional["StrCmp"] = None,
        ends_with: Optional["StrCmp"] = None,
    ) -> "NQ":
        self.set_str_property_filter(
            "ip_address",
            _str_cmps("ip_address", eq=eq, contains=contains, ends_with=ends_with),
        )
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

    def with_ip_connections(
        self: "NQ", ip_connections_query: Optional["IIpConnectionQuery"] = None
    ) -> "NQ":
        ip_connections = ip_connections_query or IpConnectionQuery()

        self.set_forward_edge_filter("ip_connections", ip_connections)
        ip_connections.set_reverse_edge_filter(
            "~ip_connections", self, "ip_connections"
        )
        return self

    def with_network_connections_from(
        self: "NQ",
        network_connections_from_query: Optional["IIpConnectionQuery"] = None,
    ) -> "NQ":
        network_connections_from = network_connections_from_query or IpConnectionQuery()
        network_connections_from.with_inbound_connection_to(cast(IpAddressQuery, self))

        return self

    def with_bound_by(
        self: "NQ",
        bound_by_query: Optional["IProcessInboundConnectionQuery"] = None,
    ) -> "NQ":
        bound_by = bound_by_query or ProcessInboundConnectionQuery()
        bound_by.with_bound_by(cast(IpAddressQuery, self))

        return self

    def _get_node_type_name(self) -> str:
        return 'IpAddress'

IIpAddressView = TypeVar("IIpAddressView", bound="IpAddressView")


class IpAddressView(DynamicNodeView):
    def __init__(
        self,
        dgraph_client: DgraphClient,
        node_key: str,
        uid: str,
        node_type: str,
        first_seen_timestamp: Optional[int] = None,
        last_seen_timestamp: Optional[int] = None,
        ip_address: Optional[str] = None,
        ip_connections: "Optional[List[IpConnectionView]]" = None,
    ):
        super(IpAddressView, self).__init__(
            dgraph_client=dgraph_client, node_key=node_key, uid=uid, node_type=node_type
        )
        self.dgraph_client = dgraph_client
        self.node_key = node_key
        self.uid = uid
        self.node_type = node_type

        self.first_seen_timestamp = first_seen_timestamp
        self.last_seen_timestamp = last_seen_timestamp
        self.ip_address = ip_address
        self.ip_connections = ip_connections

    def get_node_type(self) -> str:
        return 'IpAddress'

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

    def get_network_connections_from(self,) -> "List[IpConnectionView]":
        return cast(
            List[IpConnectionView],
            self.fetch_edges("~inbound_connection_to", List[IpConnectionView]),
        )

    def get_bound_by(self,) -> "List[ProcessInboundConnectionView]":
        return cast(
            List[ProcessInboundConnectionView],
            self.fetch_edges("~bound_by", ProcessInboundConnectionView),
        )

    @staticmethod
    def _get_property_types() -> Mapping[str, "PropertyT"]:
        return {
            "first_seen_timestamp": int,
            "last_seen_timestamp": int,
            "ip_address": str,
        }

    @staticmethod
    def _get_forward_edge_types() -> Mapping[str, "EdgeViewT"]:
        f_edges = {
            "ip_connections": [IpConnectionView]
        }  # type: Dict[str, Optional["EdgeViewT"]]

        return cast(
            Mapping[str, "EdgeViewT"], {fe[0]: fe[1] for fe in f_edges.items() if fe[1]}
        )

    def _get_forward_edges(self) -> "Mapping[str, ForwardEdgeView]":
        f_edges = {
            "ip_connections": self.ip_connections
        }  # type: Dict[str, Optional[ForwardEdgeView]]

        return cast(
            Mapping[str, ForwardEdgeView],
            {fe[0]: fe[1] for fe in f_edges.items() if fe[1]},
        )

    def _get_properties(self, fetch: bool = False) -> Mapping[str, Union[str, int]]:
        props = {
            "first_seen_timestamp": self.first_seen_timestamp,
            "last_seen_timestamp": self.last_seen_timestamp,
            "ip_address": self.ip_address,
        }

        return {p[0]: p[1] for p in props.items() if p[1] is not None}


from grapl_analyzerlib.nodes.ip_connection_node import (
    IpConnectionView,
    IpConnectionQuery,
    IIpConnectionQuery,
)

from grapl_analyzerlib.nodes.process_inbound_network_connection import (
    ProcessInboundConnectionQuery,
    IProcessInboundConnectionQuery,
    ProcessInboundConnectionView,
)
