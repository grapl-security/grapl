from __future__ import annotations

"""
This module contains a bunch of application-specific code that should be
brought up to the analyzerlib (or some application-level) layer.
It was previously interleaved with the SerDe structs, causing other fun things
like defining a client in the same file as messages due to import cycles.
"""

import enum
from collections import defaultdict, deque
from dataclasses import dataclass, field
from typing import DefaultDict, Deque, Iterable, Iterator, Mapping, Sequence

from python_proto.api.graph_query.v1beta1.client import GraphQueryClient
from python_proto.api.graph_query.v1beta1.messages import (
    EdgeNameMap,
    EdgeQueryMap,
    GraphQuery,
    GraphView,
    MatchedGraphWithUid,
    NodePropertiesView,
    NodePropertyQuery,
    NodePropertyQueryMap,
    NoMatchWithUid,
    OrStringFilters,
    QueryId,
    StringFilter,
    UidFilter,
)
from python_proto.common import Uuid
from python_proto.grapl.common.v1beta1.messages import (
    EdgeName,
    NodeType,
    PropertyName,
    Uid,
)


@dataclass(frozen=True, slots=True)
class EdgeFilters:
    node_queries: list[NodeQuery] = field(default_factory=list)

    def append(self, node_query: NodeQuery) -> None:
        self.node_queries.append(node_query)

    def extend(self, node_queries: Iterable[NodeQuery]) -> None:
        self.node_queries.extend(node_queries)

    def __iter__(self) -> Iterator[NodeQuery]:
        return self.node_queries.__iter__()

    def __len__(self) -> int:
        return self.node_queries.__len__()


@dataclass(frozen=True, slots=True)
class NodeQuery:
    node_property_query: NodePropertyQuery
    edge_filters: DefaultDict[EdgeName, EdgeFilters] = field(
        default_factory=lambda: defaultdict(EdgeFilters)
    )
    edge_map: dict[EdgeName, EdgeName] = field(default_factory=dict)

    def get_query_id(self) -> QueryId:
        return self.node_property_query.query_id

    @property
    def string_filters(self) -> Mapping[PropertyName, OrStringFilters]:
        return self.node_property_query.string_filters

    def with_string_filters(
        self, property_name: PropertyName, filters: Sequence[StringFilter]
    ) -> NodeQuery:
        self.node_property_query.with_string_filters(property_name, filters)
        return self

    def with_uid_filters(self, filters: Sequence[UidFilter]) -> NodeQuery:
        self.node_property_query.with_uid_filters(filters)
        return self

    def with_edge_filter(
        self,
        edge_name: EdgeName,
        reverse_edge_name: EdgeName,
        edge_filter: NodeQuery,
    ) -> NodeQuery:
        self.edge_filters[edge_name].append(edge_filter)
        edge_filter.edge_filters[reverse_edge_name].append(self)

        self.edge_map[edge_name] = reverse_edge_name
        self.edge_map[reverse_edge_name] = edge_name

        edge_filter.edge_map[edge_name] = reverse_edge_name
        edge_filter.edge_map[reverse_edge_name] = edge_name

        return self

    def get_reverse_edge_name(self, edge_name: EdgeName) -> EdgeName:
        return self.edge_map[edge_name]

    def into_graph_query(self) -> GraphQuery:
        node_property_queries: NodePropertyQueryMap = NodePropertyQueryMap()
        edge_filters: EdgeQueryMap = EdgeQueryMap()
        edge_map: EdgeNameMap = EdgeNameMap()

        node_property_queries[
            self.node_property_query.query_id
        ] = self.into_node_property_query()
        for node, edge, neighbor in NodeQueryIterator(self):
            node_property_queries[
                node.node_property_query.query_id
            ] = self.into_node_property_query()
            edge_filters[(node.node_property_query.query_id, edge)].add(
                neighbor.node_property_query.query_id
            )
            edge_map.update(node.edge_map)

        return GraphQuery(
            root_query_id=self.node_property_query.query_id,
            node_property_queries=node_property_queries,
            edge_filters=edge_filters,
            edge_map=edge_map,
        )

    def into_node_property_query(self) -> NodePropertyQuery:
        return NodePropertyQuery(
            node_type=self.node_property_query.node_type,
            query_id=self.node_property_query.query_id,
            string_filters=self.node_property_query.string_filters,
        )


@dataclass(frozen=False)
class NodeQueryIterator:
    parent: NodeQuery
    query_ids: dict[NodeQuery, QueryId] = field(default_factory=dict)
    visited: set[tuple[QueryId, EdgeName, QueryId]] = field(default_factory=set)
    neighbors: Deque[NodeQuery] = field(default_factory=deque)

    def __iter__(self) -> Iterator[tuple[NodeQuery, EdgeName, NodeQuery]]:
        while True:
            for edge_name, neighbors in self.parent.edge_filters.items():
                for neighbor in neighbors:
                    if self.check_visited(self.parent, edge_name, neighbor):
                        continue
                    yield self.parent, edge_name, neighbor
                    self.add_visited(self.parent, edge_name, neighbor)
                    self.neighbors.append(neighbor)

            try:
                self.parent = self.neighbors.pop()
            except IndexError:
                return

    def check_visited(
        self, src: NodeQuery, edge_name: EdgeName, dst: NodeQuery
    ) -> bool:
        if (src.get_query_id(), edge_name, dst.get_query_id()) in self.visited:
            return True
        return False

    def add_visited(self, src: NodeQuery, edge_name: EdgeName, dst: NodeQuery) -> None:
        self.visited.add((src.get_query_id(), edge_name, dst.get_query_id()))
        self.visited.add(
            (
                dst.get_query_id(),
                src.get_reverse_edge_name(edge_name),
                src.get_query_id(),
            )
        )


@dataclass(frozen=True, slots=True)
class NodeView(NodePropertiesView):
    graph: GraphView
    graph_query_client: GraphQueryClient
    tenant_id: Uuid

    @staticmethod
    def from_parts(
        node_properties: NodePropertiesView,
        graph: GraphView,
        graph_query_client: GraphQueryClient,
        tenant_id: Uuid,
    ) -> NodeView:
        return NodeView(
            uid=node_properties.uid,
            node_type=node_properties.node_type,
            string_properties=node_properties.string_properties,
            graph=graph,
            graph_query_client=graph_query_client,
            tenant_id=tenant_id,
        )

    def get_node(self, node_uid: Uid) -> NodeView | None:
        if n := self.graph.nodes.get(node_uid):
            return NodeView.from_parts(
                n, self.graph, self.graph_query_client, tenant_id=self.tenant_id
            )
        return None

    def get_neighbors(
        self,
        src_edge_name: EdgeName,
        dst_edge_name: EdgeName,
        fetch_filter: NodeQuery,
    ) -> Iterator[NodeView]:
        # todo: Apply the filter to not just the nodes we fetch but also
        #       the nodes we have locally
        self.fetch_neighbors(src_edge_name, dst_edge_name, fetch_filter)
        for neighbor_uid in self.graph.edges.get((self.uid, src_edge_name)) or set():
            if neighbor := self.get_node(neighbor_uid):
                yield neighbor
            else:
                raise Exception("Malformed GraphView, node does not exist")

    def fetch_neighbors(
        self,
        src_edge_name: EdgeName,
        dst_edge_name: EdgeName,
        neighbor_filter: NodeQuery,
    ) -> None:
        node_query = NodeQuery(
            NodePropertyQuery(node_type=self.node_type)
        ).with_edge_filter(src_edge_name, dst_edge_name, neighbor_filter)

        graph_query = node_query.into_graph_query()

        response = self.graph_query_client.query_from_uid(
            tenant_id=self.tenant_id,
            node_uid=self.uid,
            graph_query=graph_query,
        )
        self.graph.merge(response.matched_graph)


class StringConflictResolution(enum.IntEnum):
    Immutable = 0


@dataclass(frozen=True, slots=True)
class StringPropertySchema:
    node_type: NodeType
    property_name: PropertyName
    conflict_resolution: StringConflictResolution


class EdgeCardinality(enum.IntEnum):
    ToOne = 0
    ToMany = 1


@dataclass(frozen=True, slots=True)
class EdgeSchema:
    src_edge_name: EdgeName
    dst_edge_name: EdgeName
    src_edge_type: NodeType
    dst_edge_type: NodeType
    src_dst_cardinality: EdgeCardinality
    dst_src_cardinality: EdgeCardinality


@dataclass(frozen=True, slots=True)
class NodeSchema:
    node_type: NodeType
    string_property_schemas: dict[PropertyName, StringPropertySchema] = field(
        default_factory=dict
    )
    edge_schemas: dict[EdgeName, EdgeSchema] = field(default_factory=dict)


# Find a better home than this bare function.
def query_with_uid(
    graph_query: GraphQuery,
    uid: Uid,
    client: GraphQueryClient,
    tenant_id: Uuid,
) -> NodeView | None:
    response = client.query_with_uid(
        node_uid=uid,
        graph_query=graph_query,
        tenant_id=tenant_id,
    )
    match response.maybe_match:
        case NoMatchWithUid() as inner:
            return None
        case MatchedGraphWithUid() as inner:
            self_node = inner.matched_graph.nodes.get(uid)
            if not self_node:
                raise Exception("Graph did not contain 'self'")
            return NodeView.from_parts(
                node_properties=self_node,
                graph=inner.matched_graph,
                graph_query_client=client,
            )
    raise Exception(f"Unknown variant {response.maybe_match}")
