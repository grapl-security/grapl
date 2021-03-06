from __future__ import annotations
from typing import Any, TypeVar, Set, Dict, Tuple, Optional

from grapl_analyzerlib.node_types import (
    EdgeT,
    PropType,
    PropPrimitive,
    EdgeRelationship,
)
from grapl_analyzerlib.nodes.entity import EntityQuery, EntityView, EntitySchema
from grapl_analyzerlib.queryable import with_str_prop, with_int_prop
from grapl_analyzerlib.schema import Schema
from grapl_analyzerlib.comparators import IntOrNot, StrOrNot, OneOrMany

NCQ = TypeVar("NCQ", bound="NetworkConnectionQuery")
NCV = TypeVar("NCV", bound="NetworkConnectionView")


def default_network_connection_properties():
    return {
        "src_ip_address": PropType(PropPrimitive.Str, False),
        "src_port": PropType(PropPrimitive.Str, False),
        "dst_ip_address": PropType(PropPrimitive.Str, False),
        "dst_port": PropType(PropPrimitive.Int, False),
        "created_timestamp": PropType(PropPrimitive.Int, False),
        "terminated_timestamp": PropType(PropPrimitive.Int, False),
        "last_seen_timestamp": PropType(PropPrimitive.Int, False),
    }


def default_network_connection_edges() -> Dict[str, Tuple[EdgeT, str]]:
    from grapl_analyzerlib.nodes.ip_port import IpPortSchema

    return {
        "inbound_network_connection_to": (
            EdgeT(NetworkConnectionSchema, IpPortSchema, EdgeRelationship.ManyToOne),
            "inbound_network_connections_from",
        )
    }


class NetworkConnectionSchema(EntitySchema):
    def __init__(self):
        super(NetworkConnectionSchema, self).__init__(
            default_network_connection_properties(),
            default_network_connection_edges(),
            lambda: NetworkConnectionView,
        )

    @staticmethod
    def self_type() -> str:
        return "NetworkConnection"


class NetworkConnectionQuery(EntityQuery[NCV, NCQ]):
    @with_int_prop("port")
    def with_port(
        self,
        *,
        eq: Optional[IntOrNot] = None,
        gt: Optional[IntOrNot] = None,
        ge: Optional[IntOrNot] = None,
        lt: Optional[IntOrNot] = None,
        le: Optional[IntOrNot] = None,
    ):
        pass

    @with_str_prop("ip_address")
    def with_ip_address(
        self,
        *,
        eq: Optional[StrOrNot] = None,
        contains: Optional[OneOrMany[StrOrNot]] = None,
        starts_with: Optional[StrOrNot] = None,
        ends_with: Optional[StrOrNot] = None,
        regexp: Optional[OneOrMany[StrOrNot]] = None,
        distance_lt: Optional[Tuple[str, int]] = None,
    ):
        pass

    def with_inbound_network_connection_to(self, *inbound_network_connection_to):
        return self.with_to_neighbor(
            IpPortQuery,
            "inbound_network_connection_to",
            "inbound_network_connections_from",
            inbound_network_connection_to,
        )

    @classmethod
    def node_schema(cls) -> "Schema":
        return NetworkConnectionSchema()


class NetworkConnectionView(EntityView[NCV, NCQ]):
    """
    .. list-table::
        :header-rows: 1

        * - Predicate
          - Type
          - Description
        * - node_key
          - string
          - A unique identifier for this node.
        * - created_timestamp
          - int
          - Time the network connection was created (in millis-since-epoch).
        * - terminated_timestamp
          - int
          - Time the network connection was terminated (in millis-since-epoch).
        * - last_seen_timestamp
          - int
          - Time the network connection was last seen (in millis-since-epoch)
        * - src_ip_address
          - string
          - IP Address of the network connection's source.
        * - src_port
          - string
          - Port of the network connection's source.
        * - dst_ip_address
          - string
          - IP Address of the network connection's destination.
        * - dst_port
          - string
          - Port of the network connection's destination.
    """

    queryable = NetworkConnectionQuery

    def __init__(
        self,
        uid: int,
        node_key: str,
        graph_client: Any,
        node_types: Set[str],
        port: Optional[int] = None,
        ip_address: Optional[str] = None,
        **kwargs,
    ):
        super().__init__(uid, node_key, graph_client, node_types, **kwargs)
        self.node_types = set(node_types)

        self.port = port
        self.ip_address = ip_address

    def get_port(self, cached=True):
        return self.get_int("port", cached=cached)

    def get_ip_address(self, cached=True):
        return self.get_str("ip_address", cached=cached)

    def get_inbound_network_connection_to(
        self, *inbound_network_connection_to, cached=False
    ):
        return self.get_neighbor(
            IpPortQuery,
            "inbound_network_connection_to",
            "inbound_network_connections_from",
            inbound_network_connection_to,
            cached=cached,
        )

    @classmethod
    def node_schema(cls) -> "Schema":
        return NetworkConnectionSchema()


from grapl_analyzerlib.nodes.ip_port import IpPortQuery, IpPortView

NetworkConnectionSchema().init_reverse()


class NetworkConnectionExtendsIpPortQuery(IpPortQuery):
    def with_inbound_network_connections_from(self, *inbound_network_connections_from):
        return self.with_to_neighbor(
            NetworkConnectionQuery,
            "inbound_network_connections_from",
            "inbound_network_connection_to",
            inbound_network_connections_from,
        )


class NetworkConnectionExtendsIpPortView(IpPortQuery):
    def get_inbound_network_connections_from(
        self, *inbound_network_connections_from, cached=False
    ):
        return self.get_neighbor(
            NetworkConnectionQuery,
            "inbound_network_connections_from",
            "inbound_network_connection_to",
            inbound_network_connections_from,
            cached=cached,
        )


IpPortQuery = IpPortQuery.extend_self(NetworkConnectionExtendsIpPortQuery)
IpPortView = IpPortView.extend_self(NetworkConnectionExtendsIpPortView)
