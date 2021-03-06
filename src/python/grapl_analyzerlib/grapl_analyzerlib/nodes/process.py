from __future__ import annotations
from typing import Any, TypeVar, List, Set, Dict, Tuple, Optional, Union

from grapl_analyzerlib.analyzer import OneOrMany
from grapl_analyzerlib.node_types import (
    EdgeT,
    PropType,
)
from grapl_analyzerlib.nodes.entity import EntityQuery, EntityView, EntitySchema
from grapl_analyzerlib.queryable import (
    with_str_prop,
    with_int_prop,
    with_to_neighbor,
)
from grapl_analyzerlib.schema import Schema

PQ = TypeVar("PQ", bound="ProcessQuery")
PV = TypeVar("PV", bound="ProcessView")


def default_process_edges() -> Dict[str, Tuple[EdgeT, str]]:
    from grapl_analyzerlib.nodes.process_outbound_connection import (
        ProcessOutboundConnectionSchema,
    )
    from grapl_analyzerlib.nodes.process_inbound_connection import (
        ProcessInboundConnectionSchema,
    )

    return {
        "children": (
            EdgeT(ProcessSchema, ProcessSchema, EdgeRelationship.ManyToOne),
            "parent",
        ),
        "created_connections": (
            EdgeT(
                ProcessSchema,
                ProcessOutboundConnectionSchema,
                EdgeRelationship.ManyToMany,
            ),
            "connections_from",
        ),
        "inbound_connections": (
            EdgeT(
                ProcessSchema,
                ProcessInboundConnectionSchema,
                EdgeRelationship.ManyToMany,
            ),
            "bound_by",
        ),
    }


def default_process_properties() -> Dict[str, PropType]:
    return {
        "process_name": PropType(PropPrimitive.Str, False),
        "image_name": PropType(PropPrimitive.Str, False),
        "process_id": PropType(PropPrimitive.Int, False),
        "created_timestamp": PropType(PropPrimitive.Int, False),
        "terminate_time": PropType(PropPrimitive.Int, False),
        "arguments": PropType(PropPrimitive.Str, False),
    }


class ProcessSchema(EntitySchema):
    def __init__(self):
        super(ProcessSchema, self).__init__(
            default_process_properties(), default_process_edges(), lambda: ProcessView
        )

    @staticmethod
    def self_type() -> str:
        return "Process"


class ProcessQuery(EntityQuery[PV, PQ]):
    @with_int_prop("process_id")
    def with_process_id(
        self: PQ,
        *,
        eq: Optional["IntOrNot"] = None,
        gt: Optional["IntOrNot"] = None,
        ge: Optional["IntOrNot"] = None,
        lt: Optional["IntOrNot"] = None,
        le: Optional["IntOrNot"] = None,
    ) -> PQ:
        return self

    @with_int_prop("created_timestamp")
    def with_created_timestamp(
        self: PQ,
        *,
        eq: Optional["IntOrNot"] = None,
        gt: Optional["IntOrNot"] = None,
        ge: Optional["IntOrNot"] = None,
        lt: Optional["IntOrNot"] = None,
        le: Optional["IntOrNot"] = None,
    ) -> PQ:
        return self

    @with_int_prop("terminate_time")
    def with_terminate_time(
        self: PQ,
        *,
        eq: Optional["IntOrNot"] = None,
        gt: Optional["IntOrNot"] = None,
        ge: Optional["IntOrNot"] = None,
        lt: Optional["IntOrNot"] = None,
        le: Optional["IntOrNot"] = None,
    ) -> PQ:
        return self

    @with_str_prop("process_name")
    def with_process_name(
        self: PQ,
        *,
        eq: Optional["StrOrNot"] = None,
        contains: Optional["OneOrMany[StrOrNot]"] = None,
        starts_with: Optional["StrOrNot"] = None,
        ends_with: Optional["StrOrNot"] = None,
        regexp: Optional["OneOrMany[StrOrNot]"] = None,
        distance_lt: Optional[Tuple[str, int]] = None,
    ) -> PQ:
        return self

    @with_str_prop("image_name")
    def with_image_name(
        self: PQ,
        *,
        eq: Optional["StrOrNot"] = None,
        contains: Optional["OneOrMany[StrOrNot]"] = None,
        starts_with: Optional["StrOrNot"] = None,
        ends_with: Optional["StrOrNot"] = None,
        regexp: Optional["OneOrMany[StrOrNot]"] = None,
        distance_lt: Optional[Tuple[str, int]] = None,
    ) -> PQ:
        return self

    @with_str_prop("arguments")
    def with_arguments(
        self: PQ,
        *,
        eq: Optional["StrOrNot"] = None,
        contains: Optional["OneOrMany[StrOrNot]"] = None,
        starts_with: Optional["StrOrNot"] = None,
        ends_with: Optional["StrOrNot"] = None,
        regexp: Optional["OneOrMany[StrOrNot]"] = None,
        distance_lt: Optional[Tuple[str, int]] = None,
    ) -> PQ:
        return self

    @with_to_neighbor(None, "children", "parent")
    def with_children(self: PQ, *children: PQ) -> PQ:
        return self

    @with_to_neighbor(None, "parent", "children")
    def with_parent(self: PQ, parent: PQ = None) -> PQ:
        return self

    @classmethod
    def node_schema(cls) -> Schema:
        return ProcessSchema()


class ProcessView(EntityView[PV, PQ]):
    """
    .. list-table::
        :header-rows: 1

        * - Predicate
          - Type
          - Description
        * - node_key
          - string
          - A unique identifier for this node.
        * - asset_id
          - string
          - A unique identifier for this asset.
        * - image_name
          - string
          - The name of the binary that was loaded for this process.
        * - process_name
          - string
          - The name of the process.
        * - arguments
          - string
          - The arguments, as passed into the process.
        * - process_id
          - int
          - The process id for this process.
        * - created_timestamp
          - int
          - Time of the process creation (in millis-since-epoch).
        * - terminate_time
          - int
          - Time of the process termination (in millis-since-epoch).
        * - children
          - List[:doc:`/nodes/process`]
          - Child processes of this process.
        * - bin_file
          - :doc:`/nodes/file`
          - The file that was executed to create this process.
        * - created_files
          - List[:doc:`/nodes/file`]
          - Files created by this process.
        * - deleted_files
          - List[:doc:`/nodes/file`]
          - Files deleted by this process.
        * - read_files
          - List[:doc:`/nodes/file`]
          - Files read by this process.
        * - wrote_files
          - List[:doc:`/nodes/file`]
          - Files written by this process.
        * - created_connections
          - List[:doc:`/nodes/process_inbound_connection`]
          - Outbound connections created by this process.
        * - inbound_connections
          - List[:doc:`/nodes/process_inbound_connection`]
          - Inbound connections created by this process.
    """

    queryable = ProcessQuery

    def __init__(
        self,
        uid: int,
        node_key: str,
        graph_client: Any,
        node_types: Set[str],
        process_name: Optional[str] = None,
        image_name: Optional[str] = None,
        process_id: Optional[int] = None,
        created_timestamp: Optional[int] = None,
        terminate_time: Optional[int] = None,
        arguments: Optional[str] = None,
        children: Optional[List["ProcessView"]] = None,
        parent: Optional["ProcessView"] = None,
        **kwargs,
    ):
        super().__init__(uid, node_key, graph_client, node_types, **kwargs)
        self.set_predicate("node_types", node_types)
        self.set_predicate("process_name", process_name)
        self.set_predicate("image_name", image_name)
        self.set_predicate("process_id", process_id)
        self.set_predicate("created_timestamp", created_timestamp)
        self.set_predicate("terminate_time", terminate_time)
        self.set_predicate("arguments", arguments)
        self.set_predicate("children", children or [])
        self.set_predicate("parent", parent)

    def get_image_name(self, cached=True) -> Optional[str]:
        return self.get_str("image_name", cached=cached)

    def get_process_name(self, cached=True) -> Optional[str]:
        if cached and self.process_name:
            return self.process_name

        self_node = (
            self.queryable()
            .with_node_key(eq=self.node_key)
            .with_process_name()
            .query_first(self.graph_client)
        )

        if self_node:
            self.process_name = self_node.process_name
        return self.process_name

    def get_process_id(self, cached=True) -> Optional[int]:
        if cached and self.process_id:
            return self.process_id

        self_node = (
            self.queryable()
            .with_node_key(eq=self.node_key)
            .with_process_id()
            .query_first(self.graph_client)
        )

        if self_node:
            self.process_id = self_node.process_id
        return self.process_id

    def get_created_timestamp(self, cached=True) -> Optional[int]:
        if cached and self.created_timestamp:
            return self.created_timestamp

        self_node = (
            self.queryable()
            .with_node_key(eq=self.node_key)
            .with_created_timestamp()
            .query_first(self.graph_client)
        )

        if self_node:
            self.created_timestamp = self_node.created_timestamp
        return self.created_timestamp

    def get_terminate_time(self, cached=True) -> Optional[int]:
        if cached and self.terminate_time:
            return self.terminate_time

        self_node = (
            self.queryable()
            .with_node_key(eq=self.node_key)
            .with_terminate_time()
            .query_first(self.graph_client)
        )

        if self_node:
            self.terminate_time = self_node.terminate_time
        return self.terminate_time

    def get_arguments(self, cached=True) -> Optional[str]:
        if cached and self.arguments:
            return self.arguments

        self_node = (
            self.queryable()
            .with_node_key(eq=self.node_key)
            .with_arguments()
            .query_first(self.graph_client)
        )

        if self_node:
            self.arguments = self_node.arguments
        return self.arguments

    def get_parent(self, parent=None, cached=True) -> Optional[str]:
        if cached and self.parent:
            return self.parent

        self_node = (
            self.queryable()
            .with_node_key(eq=self.node_key)
            .with_parent(parent)
            .query_first(self.graph_client)
        )

        if self_node:
            self.parent = self_node.parent
        return self.parent

    def get_children(self, *children: ProcessQuery, cached=True) -> "List[ProcessView]":
        if cached and self.children:
            return self.children

        self_node = (
            self.queryable()
            .with_node_key(eq=self.node_key)
            .with_children(children)
            .query_first(self.graph_client)
        )

        if self_node:
            self.children = self_node.children
        return self.children

    @classmethod
    def node_schema(cls) -> "Schema":
        return ProcessSchema()


from grapl_analyzerlib.comparators import IntOrNot, StrOrNot

from grapl_analyzerlib.nodes.process_outbound_connection import *
from grapl_analyzerlib.nodes.process_inbound_connection import *


class ProcessExtendsProcessOutboundConnectionQuery(ProcessOutboundConnectionQuery):
    def with_connections_from(self, *connections_from: ProcessQuery):
        connections_from = connections_from or [ProcessQuery()]
        self.set_neighbor_filters("connections_from", connections_from)

        for connection in connections_from:
            connection.set_neighbor_filters("created_connections", [self])

        return self


class ProcessExtendsProcessOutboundConnectionView(ProcessOutboundConnectionView):
    def __init__(
        self,
        uid: int,
        node_key: str,
        graph_client: Any,
        node_types: Set[str],
        connections_from: Optional[List[ProcessView]] = None,
        **kwargs,
    ):
        super().__init__(
            uid=uid,
            node_key=node_key,
            graph_client=graph_client,
            node_types=node_types,
            **kwargs,
        )

        self.connections_from = connections_from or []

    def get_connections_from(self, *connections_from: ProcessQuery, cached=False):
        if cached and self.connections_from:
            return self.connections_from

        self_node = (
            self.queryable()
            .with_node_key(eq=self.node_key)
            .with_connections_from(*connections_from)
            .query_first(self.graph_client)
        )

        if self_node:
            self.connections_from = self_node.connections_from
        return self.connections_from


class ProcessExtendsProcessInboundConnectionQuery(ProcessInboundConnectionQuery):
    def with_bound_by(self, *bound_by: ProcessQuery):
        bound_by = bound_by or [ProcessQuery()]
        self.set_neighbor_filters("bound_by", bound_by)

        for binder in bound_by:
            binder.set_neighbor_filters("inbound_connections", [self])

        return self


class ProcessExtendsProcessInboundConnectionView(ProcessInboundConnectionView):
    def __init__(
        self,
        uid: int,
        node_key: str,
        graph_client: Any,
        node_types: Set[str],
        bound_by: Optional[List[ProcessView]] = None,
        **kwargs,
    ):
        super().__init__(
            uid=uid,
            node_key=node_key,
            graph_client=graph_client,
            node_types=node_types,
            **kwargs,
        )

        self.bound_by = bound_by or []

    def get_bound_by(self, *bound_by: ProcessQuery, cached=False):
        if cached and self.bound_by:
            return self.bound_by

        self_node = (
            self.queryable()
            .with_node_key(eq=self.node_key)
            .with_bound_by(*bound_by)
            .query_first(self.graph_client)
        )

        if self_node:
            self.bound_by = self_node.bound_by

        return self.bound_by


ProcessSchema().init_reverse()

ProcessOutboundConnectionQuery = ProcessOutboundConnectionQuery.extend_self(
    ProcessExtendsProcessOutboundConnectionQuery
)
ProcessOutboundConnectionView = ProcessOutboundConnectionView.extend_self(
    ProcessExtendsProcessOutboundConnectionView
)

ProcessInboundConnectionQuery = ProcessInboundConnectionQuery.extend_self(
    ProcessExtendsProcessInboundConnectionQuery
)
ProcessInboundConnectionView = ProcessInboundConnectionView.extend_self(
    ProcessExtendsProcessInboundConnectionView
)


from grapl_analyzerlib.nodes.asset import (
    AssetExtendsProcessQuery,
    AssetExtendsProcessView,
)


ProcessQuery = ProcessQuery.extend_self(AssetExtendsProcessQuery)
ProcessView = ProcessView.extend_self(AssetExtendsProcessView)
