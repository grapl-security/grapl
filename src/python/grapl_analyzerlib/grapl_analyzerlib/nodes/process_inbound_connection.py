from __future__ import annotations
from typing import Any, TypeVar

from grapl_analyzerlib.node_types import (
    EdgeT,
    PropType,
    PropPrimitive,
    EdgeRelationship,
)
from grapl_analyzerlib.nodes.entity import EntityQuery, EntityView, EntitySchema
from grapl_analyzerlib.nodes.ip_address import IpAddressQuery
from grapl_analyzerlib.queryable import with_str_prop, with_int_prop
from grapl_analyzerlib.schema import Schema
from grapl_analyzerlib.comparators import IntOrNot, StrOrNot, OneOrMany

PICQ = TypeVar("PICQ", bound="ProcessInboundConnectionQuery")
PICV = TypeVar("PICV", bound="ProcessInboundConnectionView")


def default_process_inbound_connection_properties():
    return {
        "protocol": PropType(PropPrimitive.Str, False),
        "created_timestamp": PropType(PropPrimitive.Int, False),
        "terminated_timestamp": PropType(PropPrimitive.Int, False),
        "port": PropType(PropPrimitive.Int, False),
        "last_seen_timestamp": PropType(PropPrimitive.Int, False),
    }


def default_process_inbound_connection_edges() -> dict[str, tuple[EdgeT, str]]:
    from grapl_analyzerlib.nodes.ip_address import IpAddressSchema

    return {
        "bound_port": (
            EdgeT(
                ProcessInboundConnectionSchema,
                IpPortSchema,
                EdgeRelationship.ManyToMany,
            ),
            "bound_by",
        ),
        "bound_ip": (
            EdgeT(
                ProcessInboundConnectionSchema,
                IpAddressSchema,
                EdgeRelationship.ManyToMany,
            ),
            "bound_ports",
        ),
    }


class ProcessInboundConnectionSchema(EntitySchema):
    def __init__(self):
        super().__init__(
            default_process_inbound_connection_properties(),
            default_process_inbound_connection_edges(),
            lambda: ProcessInboundConnectionView,
        )

    @staticmethod
    def self_type() -> str:
        return "ProcessInboundConnection"


class ProcessInboundConnectionQuery(EntityQuery[PICV, PICQ]):
    @with_str_prop("protocol")
    def with_protocol(
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

    @with_int_prop("created_timestamp")
    def with_created_timestamp(
        self,
        *,
        eq: IntOrNot | None = None,
        gt: IntOrNot | None = None,
        ge: IntOrNot | None = None,
        lt: IntOrNot | None = None,
        le: IntOrNot | None = None,
    ):
        pass

    @with_int_prop("terminated_timestamp")
    def with_terminated_timestamp(
        self,
        *,
        eq: IntOrNot | None = None,
        gt: IntOrNot | None = None,
        ge: IntOrNot | None = None,
        lt: IntOrNot | None = None,
        le: IntOrNot | None = None,
    ):
        pass

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

    def with_bound_port(self, *ip_ports):
        return self.with_to_neighbor(IpPortQuery, "bound_port", "bound_by", ip_ports)

    def with_bound_ip(self, *bound_ips):
        return self.with_to_neighbor(
            IpAddressQuery, "bound_ip", "bound_ports", bound_ips
        )

    @classmethod
    def node_schema(cls) -> Schema:
        return ProcessInboundConnectionSchema()


class ProcessInboundConnectionView(EntityView[PICV, PICQ]):
    """
    .. list-table::
        :header-rows: 1

        * - Predicate
          - Type
          - Description
        * - node_key
          - string
          - A unique identifier for this node
        * - created_timestamp
          - int
          - Time the process inbound network connection was created (in millis-since-epoch).
        * - terminated_timestamp
          - int
          - Time the process inbound network connection was terminated (in millis-since-epoch).
        * - last_seen_timestamp
          - int
          - Time the process inbound network connection was last seen (in millis-since-epoch)
        * - port
          - int
          - Port of the inbound process network connection.
        * - ip_address
          - str
          - IP Address of the inbound process network connection.
        * - protocol
          - int
          - Network protocol of the inbound process network connection.
        * - bound_port
          - List[:doc:`/nodes/ip_port`]
          - todo: documentation
        * - bound_by
          - List[:doc:`/nodes/process`]
          - todo: documentation
    """

    queryable = ProcessInboundConnectionQuery

    def __init__(
        self,
        uid: int,
        node_key: str,
        graph_client: Any,
        node_types: set[str],
        created_timestamp: int | None = None,
        terminated_timestamp: int | None = None,
        last_seen_timestamp: int | None = None,
        port: int | None = None,
        ip_address: str | None = None,
        protocol: str | None = None,
        **kwargs,
    ):
        super().__init__(uid, node_key, graph_client, **kwargs)
        self.node_types = set(node_types)

        self.created_timestamp = created_timestamp
        self.terminated_timestamp = terminated_timestamp
        self.last_seen_timestamp = last_seen_timestamp
        self.port = port
        self.ip_address = ip_address
        self.protocol = protocol

    def get_protocol(self, cached=True):
        self.get_str("protocol", cached=cached)

    def get_created_timestamp(self, cached=True):
        self.get_int("created_timestamp", cached=cached)

    def get_terminated_timestamp(self, cached=True):
        self.get_int("terminated_timestamp", cached=cached)

    def get_port(self, cached=True):
        self.get_int("port", cached=cached)

    def get_last_seen_timestamp(self, cached=True):
        self.get_int("last_seen_timestamp", cached=cached)

    def get_bound_port(self, *ip_ports, cached=False):
        return self.get_neighbor(
            IpPortQuery, "bound_port", "bound_by", ip_ports, cached=cached
        )

    def get_bound_ip(self, *bound_ips, cached=False):
        return self.get_neighbor(
            IpAddressQuery, "bound_ip", "bound_ports", bound_ips, cached=cached
        )

    @classmethod
    def node_schema(cls) -> Schema:
        return ProcessInboundConnectionSchema()


from grapl_analyzerlib.nodes.ip_port import IpPortSchema, IpPortQuery, IpPortView

ProcessInboundConnectionSchema().init_reverse()


class ProcessInboundConnectionExtendsIpPortQuery(IpPortQuery):
    def with_bound_port(self, *bound_ports):
        self.with_to_neighbor(
            ProcessInboundConnectionQuery, "bound_port", "bound_by", bound_ports
        )


class ProcessInboundConnectionExtendsIpPortView(IpPortView):
    def get_bound_port(self, *bound_ports, cached=False):
        self.get_neighbor(
            ProcessInboundConnectionQuery,
            "bound_port",
            "bound_by",
            bound_ports,
            cached=cached,
        )


IpPortQuery = IpPortQuery.extend_self(ProcessInboundConnectionExtendsIpPortQuery)
IpPortView = IpPortView.extend_self(ProcessInboundConnectionExtendsIpPortView)
