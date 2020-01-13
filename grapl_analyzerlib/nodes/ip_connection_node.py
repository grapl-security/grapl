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

IIpConnectionQuery = TypeVar("IIpConnectionQuery", bound="IpConnectionQuery")


class IpConnectionQuery(DynamicNodeQuery):
    def __init__(self):
        super(IpConnectionQuery, self).__init__("IpConnection", IpConnectionView)
        self._created_timestamp = []  # type: List[List[Cmp[int]]]
        self._terminated_timestamp = []  # type: List[List[Cmp[int]]]
        self._last_seen_timestamp = []  # type: List[List[Cmp[int]]]

        self._src_ip_address = []  # type: List[List[Cmp[str]]]
        self._src_port = []  # type: List[List[Cmp[str]]]
        self._dst_ip_address = []  # type: List[List[Cmp[str]]]
        self._dst_port = []  # type: List[List[Cmp[str]]]

        self._inbound_connection_to = None  # type: Optional[IIpAddressQuery]

    def with_src_ip_address(
        self,
        eq: Optional[StrCmp] = None,
        contains: Optional[StrCmp] = None,
        ends_with: Optional[StrCmp] = None,
        starts_with: Optional[StrCmp] = None,
        regexp: Optional[StrCmp] = None,
        distance: Optional[Tuple[StrCmp, int]] = None,
    ) -> "NQ":
        self.set_str_property_filter(
            "src_ip_address",
            _str_cmps(
                "src_ip_address",
                eq=eq,
                contains=contains,
                ends_with=ends_with,
                starts_with=starts_with,
                regexp=regexp,
                distance=distance,
            ),
        )
        return self

    def with_src_port(
        self,
        eq: Optional["StrCmp"] = None,
        contains: Optional["StrCmp"] = None,
        ends_with: Optional["StrCmp"] = None,
    ) -> "NQ":
        self.set_str_property_filter(
            "src_port",
            _str_cmps("src_port", eq=eq, contains=contains, ends_with=ends_with),
        )
        return self

    def with_dst_ip_address(
        self,
        eq: Optional[StrCmp] = None,
        contains: Optional[StrCmp] = None,
        ends_with: Optional[StrCmp] = None,
        starts_with: Optional[StrCmp] = None,
        regexp: Optional[StrCmp] = None,
        distance: Optional[Tuple[StrCmp, int]] = None,
    ) -> "NQ":
        self.set_str_property_filter(
            "dst_ip_address",
            _str_cmps(
                "dst_ip_address",
                eq=eq,
                contains=contains,
                ends_with=ends_with,
                starts_with=starts_with,
                regexp=regexp,
                distance=distance,
            ),
        )
        return self

    def with_dst_port(
        self,
        eq: Optional["StrCmp"] = None,
        contains: Optional["StrCmp"] = None,
        ends_with: Optional["StrCmp"] = None,
    ) -> "NQ":
        self.set_str_property_filter(
            "dst_port",
            _str_cmps("dst_port", eq=eq, contains=contains, ends_with=ends_with),
        )
        return self

    def with_created_timestamp(
        self: "NQ",
        eq: Optional["IntCmp"] = None,
        gt: Optional["IntCmp"] = None,
        lt: Optional["IntCmp"] = None,
    ) -> "NQ":
        self.set_int_property_filter(
            "created_timestamp", _int_cmps("created_timestamp", eq=eq, gt=gt, lt=lt)
        )
        return self

    def with_terminated_timestamp(
        self: "NQ",
        eq: Optional["IntCmp"] = None,
        gt: Optional["IntCmp"] = None,
        lt: Optional["IntCmp"] = None,
    ) -> "NQ":
        self.set_int_property_filter(
            "terminated_timestamp",
            _int_cmps("terminated_timestamp", eq=eq, gt=gt, lt=lt),
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

    def with_inbound_connection_to(
        self: "NQ", inbound_connection_to_query: Optional["IIpAddressQuery"] = None
    ) -> "NQ":
        inbound_connection_to = inbound_connection_to_query or IpAddressQuery()

        self.set_forward_edge_filter("inbound_connection_to", inbound_connection_to)
        inbound_connection_to.set_reverse_edge_filter(
            "~inbound_connection_to", self, "inbound_connection_to"
        )
        return self

    def with_connecting_ips(
        self: "NQ", connecting_ips_query: Optional["IIpAddressQuery"] = None
    ) -> "NQ":
        connecting_ips = connecting_ips_query or IpAddressQuery()
        connecting_ips.with_ip_connections(cast(IpConnectionQuery, self))

        return self

    def _get_node_type_name(self) -> str:
        return 'IpConnection'

IIpConnectionView = TypeVar("IIpConnectionView", bound="IpConnectionView")


class IpConnectionView(DynamicNodeView):
    def __init__(
        self,
        dgraph_client: DgraphClient,
        node_key: str,
        uid: str,
        node_type: str,
        created_timestamp: Optional[int] = None,
        terminated_timestamp: Optional[int] = None,
        last_seen_timestamp: Optional[int] = None,
        src_ip_address: Optional[str] = None,
        src_port: Optional[str] = None,
        dst_ip_address: Optional[str] = None,
        dst_port: Optional[str] = None,
        inbound_connection_to: "Optional[IpAddressView]" = None,
    ):
        super(IpConnectionView, self).__init__(
            dgraph_client=dgraph_client, node_key=node_key, uid=uid, node_type=node_type
        )
        self.dgraph_client = dgraph_client
        self.node_key = node_key
        self.uid = uid
        self.node_type = node_type

        self.created_timestamp = created_timestamp
        self.terminated_timestamp = terminated_timestamp
        self.last_seen_timestamp = last_seen_timestamp
        self.src_ip_address = src_ip_address
        self.src_port = src_port
        self.dst_ip_address = dst_ip_address
        self.dst_port = dst_port
        self.inbound_connection_to = inbound_connection_to


    def get_node_type(self) -> str:
        return 'IpConnection'

    def get_created_timestamp(self) -> Optional[int]:
        if not self.created_timestamp:
            self.created_timestamp = cast(
                Optional[int], self.fetch_property("created_timestamp", int)
            )
        return self.created_timestamp

    def get_terminated_timestamp(self) -> Optional[int]:
        if not self.terminated_timestamp:
            self.terminated_timestamp = cast(
                Optional[int], self.fetch_property("terminated_timestamp", int)
            )
        return self.terminated_timestamp

    def get_last_seen_timestamp(self) -> Optional[int]:
        if not self.last_seen_timestamp:
            self.last_seen_timestamp = cast(
                Optional[int], self.fetch_property("last_seen_timestamp", int)
            )
        return self.last_seen_timestamp

    def get_src_ip_address(self) -> Optional[str]:
        if not self.src_ip_address:
            self.src_ip_address = cast(
                Optional[str], self.fetch_property("src_ip_address", str)
            )
        return self.src_ip_address

    def get_src_port(self) -> Optional[str]:
        if not self.src_port:
            self.src_port = cast(Optional[str], self.fetch_property("src_port", str))
        return self.src_port

    def get_dst_ip_address(self) -> Optional[str]:
        if not self.dst_ip_address:
            self.dst_ip_address = cast(
                Optional[str], self.fetch_property("dst_ip_address", str)
            )
        return self.dst_ip_address

    def get_dst_port(self) -> Optional[str]:
        if not self.dst_port:
            self.dst_port = cast(Optional[str], self.fetch_property("dst_port", str))
        return self.dst_port

    def get_connecting_ips(self,) -> "List[IpAddressView]":
        return cast(
            List[IpAddressView], self.fetch_edges("~ip_connections", IpAddressView)
        )

    @staticmethod
    def _get_property_types() -> Mapping[str, "PropertyT"]:
        return {
            "created_timestamp": int,
            "terminated_timestamp": int,
            "last_seen_timestamp": int,
            "src_ip_address": str,
            "src_port": str,
            "dst_ip_address": str,
            "dst_port": str,
        }

    @staticmethod
    def _get_forward_edge_types() -> Mapping[str, "EdgeViewT"]:
        f_edges = {
            "inbound_connection_to": IpAddressView
        }  # type: Dict[str, Optional["EdgeViewT"]]

        return cast(
            Mapping[str, "EdgeViewT"], {fe[0]: fe[1] for fe in f_edges.items() if fe[1]}
        )

    def _get_forward_edges(self) -> "Mapping[str, ForwardEdgeView]":
        f_edges = {
            "inbound_connection_to": self.inbound_connection_to
        }  # type: Dict[str, Optional[ForwardEdgeView]]

        return cast(
            Mapping[str, ForwardEdgeView],
            {fe[0]: fe[1] for fe in f_edges.items() if fe[1]},
        )

    def _get_properties(self, fetch: bool = False) -> Mapping[str, Union[str, int]]:
        props = {
            "created_timestamp": self.created_timestamp,
            "terminated_timestamp": self.terminated_timestamp,
            "last_seen_timestamp": self.last_seen_timestamp,
            "src_ip_address": self.src_ip_address,
            "src_port": self.src_port,
            "dst_ip_address": self.dst_ip_address,
            "dst_port": self.dst_port,
        }

        return {p[0]: p[1] for p in props.items() if p[1] is not None}


# def main():
#     schema = IpConnectionSchema()
#
#     query = generate_plugin_query(schema)
#     view = generate_plugin_view(schema)
#     query_extensions = generate_plugin_query_extensions(schema)
#     view_extensions = generate_plugin_view_extensions(schema)
#
#     print(query)
#     print(view)
#     print(query_extensions)
#     print(view_extensions)
#

from grapl_analyzerlib.nodes.ip_address_node import (
    IIpAddressQuery,
    IpAddressQuery,
    IpAddressView,
)

from grapl_analyzerlib.schemas.schema_builder import (
    generate_plugin_view_extensions,
    generate_plugin_query_extensions,
    generate_plugin_view,
    generate_plugin_query,
)


# if __name__ == '__main__':
#     main()
