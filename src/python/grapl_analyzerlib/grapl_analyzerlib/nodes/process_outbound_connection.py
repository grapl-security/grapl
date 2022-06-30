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

POCQ = TypeVar("POCQ", bound="ProcessOutboundConnectionQuery")
POCV = TypeVar("POCV", bound="ProcessOutboundConnectionView")


def default_process_outbound_connection_properties():
    return {
        "created_timestamp": PropType(PropPrimitive.Int, False),
        "terminated_timestamp": PropType(PropPrimitive.Int, False),
        "last_seen_timestamp": PropType(PropPrimitive.Int, False),
        "port": PropType(PropPrimitive.Int, False),
        "ip_address": PropType(PropPrimitive.Str, False),
        "protocol": PropType(PropPrimitive.Str, False),
    }


def default_process_outbound_connection_edges() -> dict[str, tuple[EdgeT, str]]:
    return {
        "connected_over": (
            # The IP + Port that was connected to
            EdgeT(
                ProcessOutboundConnectionSchema,
                IpPortSchema,
                EdgeRelationship.ManyToOne,
            ),
            "process_connections",
        ),
        "connected_to": (
            # The IP + Port that was connected to
            EdgeT(
                ProcessOutboundConnectionSchema,
                IpPortSchema,
                EdgeRelationship.ManyToOne,
            ),
            "ip_port_connections_from",
        ),
    }


class ProcessOutboundConnectionSchema(EntitySchema):
    def __init__(self):
        super().__init__(
            default_process_outbound_connection_properties(),
            default_process_outbound_connection_edges(),
            lambda: ProcessOutboundConnectionView,
        )

    @staticmethod
    def self_type() -> str:
        return "ProcessOutboundConnection"


class ProcessOutboundConnectionQuery(EntityQuery[POCV, POCQ]):
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

    @classmethod
    def node_schema(cls) -> Schema:
        return ProcessOutboundConnectionSchema()


class ProcessOutboundConnectionView(EntityView[POCV, POCQ]):
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
          - Time the process outbound network connection was created (in millis-since-epoch).
        * - terminated_timestamp
          - int
          - Time the process outbound network connection was terminated (in millis-since-epoch).
        * - last_seen_timestamp
          - int
          - Time the process outbound network connection was last seen (in millis-since-epoch)
        * - port
          - int
          - Port of the outbound process network connection.
        * - ip_address
          - str
          - IP Address of the outbound process network connection.
        * - protocol
          - int
          - Network protocol of the outbound process network connection.
        * - connecting_processes
          - :doc:`/nodes/process`
          - todo: documentation
        * - connected_over
          - :doc:`/nodes/ip_port`
          - todo: documentation
        * - connected_to
          - :doc:`/nodes/ip_port`
          - todo: documentation
    """

    queryable = ProcessOutboundConnectionQuery

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
        super().__init__(uid, node_key, graph_client, node_types, **kwargs)
        self.node_types = set(node_types)

        self.created_timestamp = created_timestamp
        self.terminated_timestamp = terminated_timestamp
        self.last_seen_timestamp = last_seen_timestamp
        self.port = port
        self.ip_address = ip_address
        self.protocol = protocol

    def get_ip_address(self, cached=True):
        self.get_str("ip_address", cached=cached)

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

    @classmethod
    def node_schema(cls) -> Schema:
        return ProcessOutboundConnectionSchema()


from grapl_analyzerlib.nodes.ip_port import IpPortSchema, IpPortQuery, IpPortView

ProcessOutboundConnectionSchema().init_reverse()


class ProcessOutboundConnectionExtendsIpPortQuery(IpPortQuery):
    def with_connected_over(self, connected_over):
        self.with_to_neighbor(
            ProcessOutboundConnectionQuery,
            "connected_over",
            "process_connections",
            connected_over,
        )

    def with_connected_to(self, connected_to):
        self.with_to_neighbor(
            ProcessOutboundConnectionQuery,
            "connected_to",
            "ip_port_connections_from",
            connected_to,
        )


class ProcessOutboundConnectionExtendsIpPortView(IpPortView):
    def get_connected_over(self, connected_over, cached=False):
        self.get_neighbor(
            ProcessOutboundConnectionQuery,
            "connected_over",
            "process_connections",
            connected_over,
            cached=cached,
        )

    def get_connected_to(self, connected_to, cached=False):
        self.get_neighbor(
            ProcessOutboundConnectionQuery,
            "connected_to",
            "ip_port_connections_from",
            connected_to,
            cached=cached,
        )


IpPortQuery = IpPortQuery.extend_self(ProcessOutboundConnectionExtendsIpPortQuery)
IpPortView = IpPortView.extend_self(ProcessOutboundConnectionExtendsIpPortView)
