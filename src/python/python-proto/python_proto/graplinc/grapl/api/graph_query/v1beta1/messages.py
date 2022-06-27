from __future__ import annotations

import os

from collections import defaultdict, deque
from enum import Enum
from typing import (
    List,
    Tuple,
    DefaultDict,
    NewType,
    Sequence,
    Iterator,
    Set,
    Deque,
    Optional,
    Dict, Mapping, Final, Generator, Any, Type, TypeVar,
)
from dataclasses import dataclass, field

import enum

import grpc

from graplinc.grapl.api.graph_query_service.v1beta1.graph_query_service_pb2 import (
    IntegerProperty as IntegerPropertyProto,
    IntFilter as IntFilterProto,
    AndIntFilters as AndIntFiltersProto,
    OrIntFilters as OrIntFiltersProto,
    StringFilter as StringFilterProto,
    AndStringFilters as AndStringFiltersProto,
    OrStringFilters as OrStringFiltersProto,
    UidFilter as UidFilterProto,
    UidFilters as UidFiltersProto,
    QueryId as QueryIdProto,
    NodePropertyQuery as NodePropertyQueryProto,
    EdgeQueryEntry as EdgeQueryEntryProto,
    EdgeQueryMap as EdgeQueryMapProto,
    EdgeMapEntry as EdgeMapEntryProto,
    EdgeMap as EdgeMapProto,
    GraphQuery as GraphQueryProto,
    NodeView as NodeViewProto,
    EdgeView as EdgeViewProto,
    EdgeViews as EdgeViewsProto,
    NodeViewEntry as NodeViewEntryProto,
    NodeViewMap as NodeViewMapProto,
    EdgeViewEntry as EdgeViewEntryProto,
    EdgeViewMap as EdgeViewMapProto,
    GraphView as GraphViewProto,
    QueryGraphWithNodeRequest as QueryGraphWithNodeRequestProto,
    QueryGraphWithNodeResponse as QueryGraphWithNodeResponseProto,
    QueryGraphFromNodeRequest as QueryGraphFromNodeRequestProto,
    QueryGraphFromNodeResponse as QueryGraphFromNodeResponseProto,
)

from graplinc.grapl.common.v1beta1.types_pb2 import (
    Uid as UidProto,
    NodeType as NodeTypeProto,
    EdgeName as EdgeNameProto,
    PropertyName as PropertyNameProto,
)

from python_proto import SerDe, P
from python_proto.common import Uuid
from python_proto.graplinc.grapl.api.graph_query.v1beta1.client import GraphQueryClient


class InvalidUid(ValueError):
    ...


class InvalidNodeType(ValueError):
    ...


class InvalidPropertyName(ValueError):
    ...


class InvalidEdgeName(ValueError):
    ...


@dataclass(frozen=True)
class QueryId(SerDe[QueryIdProto]):
    value: int = field(default_factory=lambda: int.from_bytes(os.urandom(8), "little") | 1)

    def __hash__(self) -> int:
        return self.value

    @staticmethod
    def deserialize(bytes_: bytes) -> QueryId:
        msg = QueryIdProto()
        return QueryId.from_proto(msg.FromString(bytes_))

    @staticmethod
    def from_proto(proto: QueryIdProto) -> QueryId:
        return QueryId(
            value=proto.value,
        )

    def into_proto(self) -> QueryIdProto:
        msg = QueryIdProto()
        msg.value = self.value

        return msg


@dataclass(frozen=True)
class Uid(SerDe[UidProto]):
    value: int

    def __post_init__(self) -> None:
        if self.value == 0:
            raise InvalidUid("Invalid uid - must not be zero")
        if self.value < 0:
            raise InvalidUid("Invalid uid - must not be negative")

    @staticmethod
    def deserialize(bytes_: bytes) -> Uid:
        msg = UidProto()
        return Uid.from_proto(msg.FromString(bytes_))

    @staticmethod
    def from_proto(proto: UidProto) -> Uid:
        return Uid(value=proto.value)

    def into_proto(self) -> UidProto:
        pass

    def __hash__(self) -> int:
        return self.value


@dataclass(frozen=True)
class NodeType(SerDe[NodeTypeProto]):
    value: str
    skip_check: bool = False

    def __post_init__(self) -> None:
        if self.skip_check:
            return
        # Check if valid
        if not self.value:
            raise InvalidNodeType("NodeType must not be empty")
        if len(self.value) > 32:
            raise InvalidNodeType("NodeType must not be longer than 32 characters")

    @staticmethod
    def deserialize(bytes_: bytes) -> NodeType:
        msg = NodeTypeProto()
        return NodeType.from_proto(msg.FromString(bytes_))

    @staticmethod
    def from_proto(proto: NodeTypeProto) -> NodeType:
        return NodeType(
            value=proto.value,
        )

    def into_proto(self) -> NodeTypeProto:
        msg = NodeTypeProto()
        msg.value = self.value

        return msg


@dataclass(frozen=True)
class PropertyName(SerDe[PropertyNameProto]):
    value: str
    skip_check: bool = False

    def __post_init__(self) -> None:
        if self.skip_check:
            return
        # Check if valid
        if not self.value:
            raise InvalidPropertyName("PropertyName must not be empty")
        if len(self.value) > 32:
            raise InvalidPropertyName(
                "PropertyName must not be longer than 32 characters"
            )

    @staticmethod
    def deserialize(bytes_: bytes) -> PropertyName:
        msg = PropertyNameProto()
        return PropertyName.from_proto(msg.FromString(bytes_))

    @staticmethod
    def from_proto(proto: PropertyNameProto) -> PropertyName:
        return PropertyName(
            value=proto.value,
        )

    def into_proto(self) -> PropertyNameProto:
        msg = PropertyNameProto()
        msg.value = self.value

        return msg


@dataclass(frozen=True)
class EdgeName(SerDe[EdgeNameProto]):
    value: str
    skip_check: bool = False

    def __post_init__(self) -> None:
        if self.skip_check:
            return
        # Check if valid
        if not self.value:
            raise InvalidEdgeName("EdgeName must not be empty")
        if len(self.value) > 32:
            raise InvalidEdgeName("EdgeName must not be longer than 32 characters")

    @staticmethod
    def deserialize(bytes_: bytes) -> EdgeName:
        msg = EdgeNameProto()
        return EdgeName.from_proto(msg.FromString(bytes_))

    @staticmethod
    def from_proto(proto: EdgeNameProto) -> EdgeName:
        return EdgeName(
            value=proto.value,
        )

    def into_proto(self) -> EdgeNameProto:
        msg = EdgeNameProto()
        msg.value = self.value
        return msg


@enum.unique
class StringOperation(enum.IntEnum):
    HAS = 1
    EQUAL = 2
    CONTAINS = 3
    REGEX = 4


@dataclass(frozen=True)
class StringFilter(SerDe[StringFilterProto]):
    operation: StringOperation
    value: str
    negated: bool

    @staticmethod
    def deserialize(bytes_: bytes) -> StringFilter:
        msg = StringFilterProto()
        return StringFilter.from_proto(msg.FromString(bytes_))

    @staticmethod
    def from_proto(proto: StringFilterProto) -> StringFilter:
        return StringFilter(
            operation=StringOperation(proto.operation.V),
            value=proto.value,
            negated=proto.negated,
        )

    def into_proto(self) -> StringFilterProto:
        msg = StringFilterProto()
        msg.operation = StringFilterProto.Operation.V(int(self.operation))
        msg.value = self.value
        msg.negated = self.negated
        return msg


@dataclass(frozen=True)
class AndStringFilters(SerDe[AndStringFiltersProto]):
    string_filters: List[StringFilter] = field(default_factory=list)

    @staticmethod
    def deserialize(bytes_: bytes) -> AndStringFilters:
        msg = AndStringFiltersProto()
        return AndStringFilters.from_proto(msg.FromString(bytes_))

    @staticmethod
    def from_proto(proto: AndStringFiltersProto) -> AndStringFilters:
        return AndStringFilters(
            string_filters=[StringFilter.from_proto(p) for p in proto.string_filters]
        )

    def into_proto(self) -> AndStringFiltersProto:
        msg = AndStringFiltersProto()
        msg.string_filters.extend((p.into_proto() for p in self.string_filters))
        return msg

    def __len__(self) -> int:
        return len(self.string_filters)

    def __iter__(self) -> Iterator[StringFilter]:
        return self.string_filters.__iter__()


@dataclass(frozen=True)
class OrStringFilters(SerDe[OrStringFiltersProto]):
    and_string_filters: List[AndStringFilters] = field(default_factory=list)

    @staticmethod
    def deserialize(bytes_: bytes) -> OrStringFilters:
        msg = OrStringFiltersProto()
        return OrStringFilters.from_proto(msg.FromString(bytes_))

    @staticmethod
    def from_proto(proto: OrStringFiltersProto) -> OrStringFilters:
        return OrStringFilters(
            and_string_filters=[AndStringFilters.from_proto(p) for p in proto.and_string_filters]
        )

    def into_proto(self) -> OrStringFiltersProto:
        msg = OrStringFiltersProto()
        msg.and_string_filters.extend((p.into_proto() for p in self.and_string_filters))
        return msg

    def append(self, and_string_filters: AndStringFilters) -> None:
        self.and_string_filters.append(and_string_filters)

    def __len__(self) -> int:
        return len(self.and_string_filters)

    def __iter__(self) -> Iterator[AndStringFilters]:
        return self.and_string_filters.__iter__()



@dataclass(frozen=True)
class EdgeQueryEntry(SerDe[EdgeQueryEntryProto]):
    query_id: QueryId
    edge_name: EdgeName
    neighbor_query_ids: Set[QueryId]


    @staticmethod
    def deserialize(bytes_: bytes) -> EdgeQueryEntry:
        msg = EdgeQueryEntryProto()
        return EdgeQueryEntry.from_proto(msg.FromString(bytes_))

    @staticmethod
    def from_proto(proto: EdgeQueryEntryProto) -> EdgeQueryEntry:
        return EdgeQueryEntry(
            query_id=QueryId.from_proto(proto.query_id),
            edge_name=EdgeName.from_proto(proto.edge_name),
            neighbor_query_ids=set((QueryId.from_proto(p) for p in proto.neighbor_query_ids)),
        )

    def into_proto(self) -> EdgeQueryEntryProto:
        msg = EdgeQueryEntryProto()
        msg.query_id.CopyFrom(self.query_id.into_proto())
        msg.edge_name.CopyFrom(self.edge_name.into_proto())
        msg.neighbor_query_ids.extend((p.into_proto() for p in self.neighbor_query_ids))
        return msg


@dataclass(frozen=True)
class EdgeQueryMap(SerDe[EdgeQueryMapProto]):
    entries: DefaultDict[Tuple[QueryId, EdgeName], Set[QueryId]]


    @staticmethod
    def deserialize(bytes_: bytes) -> EdgeQueryEntry:
        msg = EdgeQueryEntryProto()
        return EdgeQueryEntry.from_proto(msg.FromString(bytes_))

    @staticmethod
    def from_proto(proto: EdgeQueryMapProto) -> EdgeQueryMap:
        entries: DefaultDict[Tuple[QueryId, EdgeName], Set[QueryId]] = defaultdict(set)
        for proto_entry in proto.entries:
            entry = EdgeQueryEntry.from_proto(proto_entry)
            entries[(entry.query_id, entry.edge_name)].update(entry.neighbor_query_ids)

        return EdgeQueryMap(entries=entries)

    def into_proto(self) -> EdgeQueryMapProto:
        msg = EdgeQueryMapProto()

        entries = (
            EdgeQueryEntry(query_id, edge_name, neighbors).into_proto()
            for ((query_id, edge_name), neighbors) in self.entries.items()
        )

        msg.entries.extend(entries)
        return msg


@dataclass
class EdgeFilters:
    node_queries: List[NodeQuery]


@dataclass
class NodeQuery(object):
    node_type: NodeType
    query_id: QueryId = field(default_factory=QueryId)
    string_filters: DefaultDict[PropertyName, OrStringFilters] = field(
        default_factory=lambda: defaultdict(OrStringFilters)
    )
    edge_filters: DefaultDict[EdgeName, EdgeFilters] = field(
        default_factory=lambda: defaultdict(EdgeFilters)
    )
    edge_map: Dict[EdgeName, EdgeName] = field(default_factory=dict)

    def with_string_filters(
            self, property_name: PropertyName, filters: Sequence[StringFilter]
    ) -> NodeQuery:
        self.string_filters[property_name].append(AndStringFilters(list(filters)))
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

    # def query_with(self, graph_query_client: ()) -> None:
    #     ...


class NodeQueryIterator(object):
    def __init__(self, node_query: NodeQuery) -> None:
        self.visited: Set[Tuple[QueryId, EdgeName, QueryId]] = set()
        self.query_ids: Dict[NodeQuery, QueryId]
        self.neighbors: Deque[NodeQuery] = deque()
        self.parent: NodeQuery = node_query

    def __iter__(self) -> Iterator[Tuple[NodeQuery, EdgeName, NodeQuery]]:
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

    def check_visited(self, src: NodeQuery, edge_name: EdgeName, dst: NodeQuery) -> bool:
        if (src.query_id, edge_name, dst.query_id) in self.visited:
            return True
        return False

    def add_visited(self, src: NodeQuery, edge_name: EdgeName, dst: NodeQuery) -> None:
        self.visited.add(
            (src.query_id, edge_name, dst.query_id)
        )
        self.visited.add(
            (dst.query_id, src.get_reverse_edge_name(edge_name), src.query_id)
        )


@dataclass(frozen=True)
class NodePropertyQuery(SerDe[NodePropertyQueryProto]):
    node_type: NodeType
    query_id: QueryId
    string_filters: DefaultDict[PropertyName, OrStringFilters] = field(
        default_factory=lambda: defaultdict(OrStringFilters)
    )

    @staticmethod
    def from_node_query(node_query: NodeQuery) -> NodePropertyQuery:
        return NodePropertyQuery(
            node_type=node_query.node_type,
            query_id=node_query.query_id,
            string_filters=node_query.string_filters,
        )



@dataclass(frozen=True)
class GraphQuery(SerDe[GraphQueryProto]):
    root_query_id: QueryId
    nodes: Dict[QueryId, NodePropertyQuery] = field(default_factory=dict)
    edges: DefaultDict[Tuple[QueryId, EdgeName], Set[QueryId]] = field(
        default_factory=lambda: defaultdict(set)
    )
    edge_map: Dict[EdgeName, EdgeName] = field(default_factory=dict)

    @staticmethod
    def deserialize(bytes_: bytes) -> GraphQuery:
        msg = GraphQueryProto()
        return GraphQuery.from_proto(msg.parse_from_string(bytes_))

    @staticmethod
    def from_proto(proto: GraphQueryProto) -> GraphQuery:
        return GraphQuery(
            root_query_id=QueryId.from_proto(proto.root_query_id),
            nodes={k: NodePropertyQuery.from_proto(v) for k, v in proto.node_property_queries().items()},
        )

    def into_proto(self) -> QueryGraphWithNodeRequestProto:
        raise NotImplementedError

    @staticmethod
    def from_node_query(root_node_query: NodeQuery) -> GraphQuery:
        nodes: Dict[QueryId, NodePropertyQuery] = dict()
        edges: DefaultDict[Tuple[QueryId, EdgeName], Set[QueryId]] = defaultdict(set)
        edge_map: Dict[EdgeName, EdgeName] = dict()

        nodes[root_node_query.query_id] = NodePropertyQuery.from_node_query(
            root_node_query
        )
        for node, edge, neighbor in NodeQueryIterator(root_node_query):
            print(node.query_id, edge, neighbor.query_id)
            nodes[node.query_id] = NodePropertyQuery.from_node_query(node)
            edges[(node.query_id, edge)].add(neighbor.query_id)
            edge_map.update(node.edge_map)

        return GraphQuery(
            root_query_id=root_node_query.query_id,
            nodes=nodes,
            edges=edges,
            edge_map=edge_map,
        )

    def query_with_uid(
            self,
            uid: Uid,
            client: GraphQueryClient,
    ) -> Optional[NodeView]:
        response = client.query_with_uid(
            QueryGraphWithNodeRequest(
                tenant_id=client.tenant_id,
                node_uid=uid,
                graph_query=self,
            )
        )
        if not response:
            return None

        return NodeView.from_parts(
            node_properties=response.matched_graph.nodes[response.root_uid],
            graph=response.matched_graph,
        )


@dataclass(frozen=True)
class GraphView:
    nodes: Dict[Uid, NodePropertiesView] = field(default_factory=dict)
    edges: Dict[Tuple[Uid, EdgeName], Set[Uid]] = field(default_factory=dict)


@dataclass(frozen=True)
class NodePropertiesView:
    uid: Uid
    node_type: NodeType
    string_properties: Dict[PropertyName, str]
    int_properties: Dict[PropertyName, int]


@dataclass(frozen=True)
class NodeView(NodePropertiesView):
    graph: GraphView

    @staticmethod
    def from_parts(node_properties: NodePropertiesView, graph: GraphView) -> NodeView:
        return NodeView(
            uid=node_properties.uid,
            node_type=node_properties.node_type,
            string_properties=node_properties.string_properties,
            int_properties=node_properties.int_properties,
            graph=graph,
        )


class StrConflictResolution(Enum):
    Immutable = 1


class Int64ConflictResolution(Enum):
    Immutable = 1
    Max = 2
    Min = 3


@dataclass(frozen=True)
class EdgeCardinality(Enum):
    ToOne = 1
    ToMany = 2


@dataclass(frozen=True)
class EdgeSchema:
    forward_edge_name: EdgeName
    reverse_edge_name: EdgeName
    forward_cardinality: EdgeCardinality
    reverse_cardinality: EdgeCardinality


@dataclass(frozen=True)
class NodeSchema:
    string_properties: Mapping[PropertyName, StrConflictResolution]
    int64_properties: Mapping[PropertyName, Int64ConflictResolution]
    edges: Mapping[EdgeName, EdgeSchema]


@dataclass(frozen=True)
class QueryGraphWithNodeRequest(SerDe[QueryGraphWithNodeRequestProto]):
    tenant_id: Uuid
    node_uid: Uid
    graph_query: GraphQuery

    @staticmethod
    def deserialize(bytes_: bytes) -> QueryGraphWithNodeRequest:
        msg = QueryGraphWithNodeRequestProto()
        return QueryGraphWithNodeRequest.from_proto(msg.parse_from_string(bytes_))

    @staticmethod
    def from_proto(proto: QueryGraphWithNodeRequestProto) -> QueryGraphWithNodeRequest:
        return QueryGraphWithNodeRequest(
            tenant_id=Uuid.from_proto(proto.tenant_id),
            node_uid=Uid.from_proto(proto.node_uid),
            graph_query=GraphQuery.from_proto(proto.graph_query)
        )

    def into_proto(self) -> QueryGraphWithNodeRequestProto:
        msg = QueryGraphWithNodeRequestProto()
        msg.graph_query.CopyFrom(self.graph_query.into_proto())
        msg.tenant_id.CopyFrom(self.tenant_id.into_proto())
        msg.node_uid.CopyFrom(self.node_uid.into_proto())

        return msg

@dataclass(frozen=True)
class QueryGraphWithNodeResponse:
    matched_graph: GraphView
    root_uid: Uid

