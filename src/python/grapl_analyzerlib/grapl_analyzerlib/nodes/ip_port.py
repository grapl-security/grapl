from __future__ import annotations
from typing import TypeVar

from grapl_analyzerlib.node_types import (
    EdgeT,
    PropType,
    PropPrimitive,
    EdgeRelationship,
)
from grapl_analyzerlib.queryable import (
    with_str_prop,
    with_int_prop,
)
from grapl_analyzerlib.schema import Schema
from grapl_analyzerlib.comparators import IntOrNot, StrOrNot
from grapl_analyzerlib.nodes.entity import EntityQuery, EntityView, EntitySchema
from grapl_analyzerlib.comparators import IntOrNot, StrOrNot, OneOrMany

IPPQ = TypeVar("IPPQ", bound="IpPortQuery")
IPPV = TypeVar("IPPV", bound="IpPortView")


def default_ip_port_properties():
    return {
        "port": PropType(PropPrimitive.Int, False),
        "ip_address": PropType(PropPrimitive.Str, False),
        "first_seen_timestamp": PropType(PropPrimitive.Int, False),
        "last_seen_timestamp": PropType(PropPrimitive.Int, False),
    }


def default_ip_port_edges() -> dict[str, tuple[EdgeT, str]]:
    from grapl_analyzerlib.nodes.network_connection import (
        NetworkConnectionSchema,
    )

    return {
        "network_connections": (
            EdgeT(IpPortSchema, NetworkConnectionSchema, EdgeRelationship.ManyToMany),
            "network_connections_from",
        )
    }


class IpPortSchema(EntitySchema):
    def __init__(self):
        super().__init__(
            default_ip_port_properties(), default_ip_port_edges(), lambda: IpPortView
        )

    @staticmethod
    def self_type() -> str:
        return "IpPort"


class IpPortQuery(EntityQuery[IPPV, IPPQ]):
    @with_int_prop("port")
    def with_port(
        self,
        *,
        eq: IntOrNot | None = None,
        gt: IntOrNot | None = None,
        ge: IntOrNot | None = None,
        lt: IntOrNot | None = None,
        le: IntOrNot | None = None,
    ):
        pass

    @with_str_prop("ip_address")
    def with_ip_address(
        self,
        *,
        eq: StrOrNot | None = None,
        contains: OneOrMany[StrOrNot] | None = None,
        starts_with: StrOrNot | None = None,
        ends_with: StrOrNot | None = None,
        regexp: OneOrMany[StrOrNot] | None = None,
        distance_lt: tuple[str, int] | None = None,
    ):
        pass

    @with_int_prop("first_seen_timestamp")
    def with_first_seen_timestamp(
        self,
        *,
        eq: IntOrNot | None = None,
        gt: IntOrNot | None = None,
        ge: IntOrNot | None = None,
        lt: IntOrNot | None = None,
        le: IntOrNot | None = None,
    ):
        pass

    @with_int_prop("last_seen_timestamp")
    def with_last_seen_timestamp(
        self,
        *,
        eq: IntOrNot | None = None,
        gt: IntOrNot | None = None,
        ge: IntOrNot | None = None,
        lt: IntOrNot | None = None,
        le: IntOrNot | None = None,
    ):
        pass

    def with_network_connections(self, *network_connections):
        return self.with_to_neighbor(
            NetworkConnectionQuery,
            "network_connections",
            "network_connections_from",
            network_connections,
        )

    @classmethod
    def node_schema(cls) -> Schema:
        return IpPortSchema()


class IpPortView(EntityView[IPPV, IPPQ]):
    """
    .. list-table::
        :header-rows: 1

        * - Predicate
          - Type
          - Description
        * - node_key
          - string
          - A unique identifier for this node.
        * - first_seen_timestamp
          - int
          - Time the IP Port was first seen (in millis-since-epoch).
        * - last_seen_timestamp
          - int
          - Time the IP Port was last seen (in millis-since-epoch).
        * - terminated_timestamp
          - int
          - Time connection was terminated (in millis-since-epoch).
        * - ip_address
          - string
          - The IP Address associated with this node. (TODO: v4? v6? both?)
        * - protocol
          - string
          - todo: documentation
        * - network_connections
          - List[:doc:`/nodes/network_connection`]
          - todo: documentation
        * - bound_by
          - List[:doc:`/nodes/process_inbound_connection`]
          - todo: documentation
        * - process_connections
          - List[:doc:`/nodes/process_outbound_connection`]
          - todo: documentation
        * - process_connects
          - List[:doc:`/nodes/process_outbound_connection`]
          - todo: documentation
    """

    queryable = IpPortQuery

    def __init__(
        self,
        uid: int,
        node_key: str,
        graph_client,
        node_types: set[str],
        port: int | None = None,
        ip_address: str | None = None,
        first_seen_timestamp: int | None = None,
        last_seen_timestamp: int | None = None,
        network_connections: list[NetworkConnectionView] | None = None,
        **kwargs,
    ):
        super().__init__(uid, node_key, graph_client, node_types, **kwargs)
        self.set_predicate("port", port)
        self.set_predicate("ip_address", ip_address)
        self.set_predicate("first_seen_timestamp", first_seen_timestamp)
        self.set_predicate("last_seen_timestamp", last_seen_timestamp)
        self.set_predicate("network_connections", network_connections or [])

    def get_port(self, cached=True):
        return self.get_int("port", cached=cached)

    def get_ip_address(self, cached=True):
        return self.get_str("ip_address", cached=cached)

    def get_first_seen_timestamp(self, cached=True):
        return self.get_int("first_seen_timestamp", cached=cached)

    def get_last_seen_timestamp(self, cached=True):
        return self.get_int("last_seen_timestamp", cached=cached)

    def get_network_connections(self, *network_connections, cached=False):
        return self.get_neighbor(
            NetworkConnectionQuery,
            "network_connections",
            "network_connections_from",
            network_connections,
            cached=cached,
        )

    @classmethod
    def node_schema(cls) -> Schema:
        return IpPortSchema()


from grapl_analyzerlib.nodes.network_connection import (
    NetworkConnectionQuery,
    NetworkConnectionView,
)


class IpPortExtendsNetworkConnectionQuery(NetworkConnectionQuery):
    def with_network_connections_from(self, *connections_from):
        self.with_to_neighbor(
            IpPortQuery,
            "network_connections_from",
            "network_connections",
            connections_from,
        )


class IpPortExtendsNetworkConnectionView(NetworkConnectionView):
    def get_network_connections_from(self, *connections_from, cached=False):
        self.get_neighbor(
            IpPortQuery,
            "network_connections_from",
            "network_connections",
            connections_from,
            cached=cached,
        )


IpPortSchema().init_reverse()

NetworkConnectionQuery = NetworkConnectionQuery.extend_self(
    IpPortExtendsNetworkConnectionQuery
)
NetworkConnectionView = NetworkConnectionView.extend_self(
    IpPortExtendsNetworkConnectionView
)
