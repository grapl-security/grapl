from __future__ import annotations

import enum
import os
from collections import defaultdict, deque
from dataclasses import InitVar, dataclass, field
from typing import (
    DefaultDict,
    Deque,
    Dict,
    Iterable,
    Iterator,
    List,
    Mapping,
    Optional,
    Sequence,
    Set,
    Tuple,
    final,
)

from graplinc.grapl.api.graph_query_service.v1beta1.graph_query_service_pb2 import (
    AndStringFilters as AndStringFiltersProto,
)
from graplinc.grapl.api.graph_query_service.v1beta1.graph_query_service_pb2 import (
    EdgeEntry as EdgeEntryProto,
)
from graplinc.grapl.api.graph_query_service.v1beta1.graph_query_service_pb2 import (
    EdgeMap as EdgeMapProto,
)
from graplinc.grapl.api.graph_query_service.v1beta1.graph_query_service_pb2 import (
    EdgeQueryEntry as EdgeQueryEntryProto,
)
from graplinc.grapl.api.graph_query_service.v1beta1.graph_query_service_pb2 import (
    EdgeQueryMap as EdgeQueryMapProto,
)
from graplinc.grapl.api.graph_query_service.v1beta1.graph_query_service_pb2 import (
    EdgeViewEntry as EdgeViewEntryProto,
)
from graplinc.grapl.api.graph_query_service.v1beta1.graph_query_service_pb2 import (
    EdgeViewMap as EdgeViewMapProto,
)
from graplinc.grapl.api.graph_query_service.v1beta1.graph_query_service_pb2 import (
    GraphQuery as GraphQueryProto,
)
from graplinc.grapl.api.graph_query_service.v1beta1.graph_query_service_pb2 import (
    GraphView as GraphViewProto,
)
from graplinc.grapl.api.graph_query_service.v1beta1.graph_query_service_pb2 import (
    NodePropertiesView as NodePropertiesViewProto,
)
from graplinc.grapl.api.graph_query_service.v1beta1.graph_query_service_pb2 import (
    NodePropertiesViewEntry as NodePropertiesViewEntryProto,
)
from graplinc.grapl.api.graph_query_service.v1beta1.graph_query_service_pb2 import (
    NodePropertiesViewMap as NodePropertiesViewMapProto,
)
from graplinc.grapl.api.graph_query_service.v1beta1.graph_query_service_pb2 import (
    NodePropertyQuery as NodePropertyQueryProto,
)
from graplinc.grapl.api.graph_query_service.v1beta1.graph_query_service_pb2 import (
    NodePropertyQueryEntry as NodePropertyQueryEntryProto,
)
from graplinc.grapl.api.graph_query_service.v1beta1.graph_query_service_pb2 import (
    NodePropertyQueryMap as NodePropertyQueryMapProto,
)
from graplinc.grapl.api.graph_query_service.v1beta1.graph_query_service_pb2 import (
    OrStringFilters as OrStringFiltersProto,
)
from graplinc.grapl.api.graph_query_service.v1beta1.graph_query_service_pb2 import (
    QueryGraphFromNodeRequest as QueryGraphFromNodeRequestProto,
)
from graplinc.grapl.api.graph_query_service.v1beta1.graph_query_service_pb2 import (
    QueryGraphFromNodeResponse as QueryGraphFromNodeResponseProto,
)
from graplinc.grapl.api.graph_query_service.v1beta1.graph_query_service_pb2 import (
    QueryGraphWithNodeRequest as QueryGraphWithNodeRequestProto,
)
from graplinc.grapl.api.graph_query_service.v1beta1.graph_query_service_pb2 import (
    QueryGraphWithNodeResponse as QueryGraphWithNodeResponseProto,
)
from graplinc.grapl.api.graph_query_service.v1beta1.graph_query_service_pb2 import (
    QueryId as QueryIdProto,
)
from graplinc.grapl.api.graph_query_service.v1beta1.graph_query_service_pb2 import (
    StringFilter as StringFilterProto,
)
from graplinc.grapl.api.graph_query_service.v1beta1.graph_query_service_pb2 import (
    StringProperties as StringPropertiesProto,
)
from graplinc.grapl.api.graph_query_service.v1beta1.graph_query_service_pb2 import (
    StringProperty as StringPropertyProto,
)
from graplinc.grapl.api.graph_query_service.v1beta1.graph_query_service_pb2 import (
    UidFilter as UidFilterProto,
)
from graplinc.grapl.api.graph_query_service.v1beta1.graph_query_service_pb2_grpc import (
    GraphQueryServiceStub,
)
from graplinc.grapl.common.v1beta1.types_pb2 import EdgeName as EdgeNameProto
from graplinc.grapl.common.v1beta1.types_pb2 import NodeType as NodeTypeProto
from graplinc.grapl.common.v1beta1.types_pb2 import PropertyName as PropertyNameProto
from graplinc.grapl.common.v1beta1.types_pb2 import Uid as UidProto
from python_proto import SerDe
from python_proto.common import Uuid


class InvalidUid(ValueError):
    ...


class InvalidNodeType(ValueError):
    ...


class InvalidPropertyName(ValueError):
    ...


class InvalidEdgeName(ValueError):
    ...


def _non_zero_int() -> int:
    """
    A helper function for returning a 64bit integer that is never
    zero
    :return: A random 64bit integer that will never be zero
    """
    return int.from_bytes(os.urandom(8), "little") | 1


@dataclass(frozen=True, slots=True)
class QueryId(SerDe[QueryIdProto]):
    value: int = field(default_factory=_non_zero_int)

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

    def __hash__(self) -> int:
        return self.value


@dataclass(frozen=True, slots=True)
@final
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


@dataclass(frozen=True, slots=True)
class NodeType(SerDe[NodeTypeProto]):
    value: str
    _skip_check: InitVar[bool] = False

    @final
    def __post_init__(self, _skip_check: bool) -> None:
        if _skip_check:
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


@dataclass(frozen=True, slots=True)
class PropertyName(SerDe[PropertyNameProto]):
    value: str
    _skip_check: InitVar[bool] = False

    @final
    def __post_init__(self, _skip_check: bool) -> None:
        if _skip_check:
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


@dataclass(frozen=True, slots=True)
class EdgeName(SerDe[EdgeNameProto]):
    value: str
    _skip_check: InitVar[bool] = False

    @final
    def __post_init__(self, _skip_check: bool) -> None:
        if _skip_check:
            return
        # todo: Stricter checks
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


@dataclass(frozen=True, slots=True)
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
            operation=StringOperation(proto.operation),
            value=proto.value,
            negated=proto.negated,
        )

    def into_proto(self) -> StringFilterProto:
        msg = StringFilterProto()
        msg.operation = StringFilterProto.Operation.V(int(self.operation))
        msg.value = self.value
        msg.negated = self.negated
        return msg


@dataclass(frozen=True, slots=True)
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


@dataclass(frozen=True, slots=True)
class OrStringFilters(SerDe[OrStringFiltersProto]):
    and_string_filters: List[AndStringFilters] = field(default_factory=list)

    @staticmethod
    def deserialize(bytes_: bytes) -> OrStringFilters:
        msg = OrStringFiltersProto()
        return OrStringFilters.from_proto(msg.FromString(bytes_))

    @staticmethod
    def from_proto(proto: OrStringFiltersProto) -> OrStringFilters:
        return OrStringFilters(
            and_string_filters=[
                AndStringFilters.from_proto(p) for p in proto.and_string_filters
            ]
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


@dataclass(frozen=True, slots=True)
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
            neighbor_query_ids=set(
                (QueryId.from_proto(p) for p in proto.neighbor_query_ids)
            ),
        )

    def into_proto(self) -> EdgeQueryEntryProto:
        msg = EdgeQueryEntryProto()
        msg.query_id.CopyFrom(self.query_id.into_proto())
        msg.edge_name.CopyFrom(self.edge_name.into_proto())
        msg.neighbor_query_ids.extend((p.into_proto() for p in self.neighbor_query_ids))
        return msg


@enum.unique
class UidOperation(enum.IntEnum):
    EQUAL = 1


@dataclass(frozen=True, slots=True)
class UidFilter(SerDe[UidFilterProto]):
    operation: UidOperation
    value: Uid

    @staticmethod
    def deserialize(bytes_: bytes) -> UidFilter:
        msg = UidFilterProto()
        return UidFilter.from_proto(msg.FromString(bytes_))

    @staticmethod
    def from_proto(proto: UidFilterProto) -> UidFilter:
        return UidFilter(
            operation=UidOperation(proto.operation),
            value=Uid.from_proto(proto.value),
        )

    def into_proto(self) -> UidFilterProto:
        msg = UidFilterProto()
        msg.operation = UidFilterProto.Operation.V(int(self.operation))
        msg.value.CopyFrom(self.value.into_proto())
        return msg


@dataclass(frozen=True, slots=True)
class EdgeQueryMap(SerDe[EdgeQueryMapProto]):
    entries: Dict[Tuple[QueryId, EdgeName], Set[QueryId]] = field(default_factory=dict)

    @staticmethod
    def deserialize(bytes_: bytes) -> EdgeQueryMap:
        msg = EdgeQueryMapProto()
        return EdgeQueryMap.from_proto(msg.FromString(bytes_))

    @staticmethod
    def from_proto(proto: EdgeQueryMapProto) -> EdgeQueryMap:
        return EdgeQueryMap(
            entries={
                (
                    QueryId.from_proto(entry.query_id),
                    EdgeName.from_proto(entry.edge_name),
                ): set((QueryId.from_proto(n) for n in entry.neighbor_query_ids))
                for entry in proto.entries
            }
        )

    def into_proto(self) -> EdgeQueryMapProto:
        msg = EdgeQueryMapProto()

        entries = (
            EdgeQueryEntry(query_id, edge_name, neighbors).into_proto()
            for ((query_id, edge_name), neighbors) in self.entries.items()
        )

        msg.entries.extend(entries)
        return msg

    def __getitem__(self, key: Tuple[QueryId, EdgeName]) -> Set[QueryId]:
        return self.entries[key]

    def __setitem__(self, key: Tuple[QueryId, EdgeName], value: Set[QueryId]) -> None:
        self.entries[key] = value


@dataclass(frozen=True, slots=True)
class NodePropertyQuery(SerDe[NodePropertyQueryProto]):
    node_type: NodeType
    query_id: QueryId = field(default_factory=QueryId)
    string_filters: DefaultDict[PropertyName, OrStringFilters] = field(
        default_factory=lambda: defaultdict(OrStringFilters)
    )
    uid_filters: List[Tuple[UidFilter, ...]] = field(default_factory=list)

    @staticmethod
    def deserialize(bytes_: bytes) -> NodePropertyQuery:
        msg = NodePropertyQueryProto()
        return NodePropertyQuery.from_proto(msg.FromString(bytes_))

    @staticmethod
    def from_proto(proto: NodePropertyQueryProto) -> NodePropertyQuery:
        return NodePropertyQuery(
            query_id=QueryId.from_proto(proto.query_id),
            node_type=NodeType.from_proto(proto.node_type),
            string_filters=defaultdict(
                OrStringFilters,
                {
                    PropertyName(p): OrStringFilters.from_proto(f)
                    for p, f in proto.string_filters.items()
                },
            ),
        )

    def into_proto(self) -> NodePropertyQueryProto:
        msg = NodePropertyQueryProto()
        msg.query_id.CopyFrom(self.query_id.into_proto())
        msg.node_type.CopyFrom(self.node_type.into_proto())
        msg.string_filters.update(
            {p.value: f.into_proto() for p, f in self.string_filters.items()}
        )
        return msg

    def with_string_filters(
        self, property_name: PropertyName, filters: Sequence[StringFilter]
    ) -> NodePropertyQuery:
        self.string_filters[property_name].append(AndStringFilters(list(filters)))
        return self

    def with_uid_filters(self, filters: Sequence[UidFilter]) -> NodePropertyQuery:
        self.uid_filters.append(tuple(filters))
        return self

    @staticmethod
    def from_node_query(node_query: NodeQuery) -> NodePropertyQuery:
        return NodePropertyQuery(
            node_type=node_query.node_property_query.node_type,
            query_id=node_query.node_property_query.query_id,
            string_filters=node_query.node_property_query.string_filters,
        )


@dataclass(frozen=True, slots=True)
class EdgeMap(SerDe[EdgeMapProto]):
    entries: Dict[EdgeName, EdgeName] = field(default_factory=dict)

    @staticmethod
    def deserialize(bytes_: bytes) -> EdgeMap:
        msg = EdgeMapProto()
        return EdgeMap.from_proto(msg.FromString(bytes_))

    @staticmethod
    def from_proto(proto: EdgeMapProto) -> EdgeMap:
        return EdgeMap(
            entries={
                EdgeName.from_proto(entry.forward_edge_name): EdgeName.from_proto(
                    entry.reverse_edge_name
                )
                for entry in proto.entries
            }
        )

    def into_proto(self) -> EdgeMapProto:
        msg = EdgeMapProto()
        msg.entries.extend(
            (
                EdgeEntryProto(
                    forward_edge_name=s.into_proto(), reverse_edge_name=d.into_proto()
                )
                for s, d in self.entries.items()
            )
        )

        return msg

    @staticmethod
    def from_entry(src_edge_name: EdgeName, dst_edge_name: EdgeName) -> EdgeMap:
        return EdgeMap(
            entries={
                src_edge_name: dst_edge_name,
                dst_edge_name: src_edge_name,
            }
        )

    def update(self, other: Mapping[EdgeName, EdgeName]) -> None:
        self.entries.update(other)

    def with_mapping(self, src_name: EdgeName, dst_name: EdgeName) -> EdgeMap:
        self.entries[src_name] = dst_name
        self.entries[dst_name] = src_name
        return self


@dataclass(frozen=True, slots=True)
class NodePropertyQueryMap(SerDe[NodePropertyQueryMapProto]):
    entries: Dict[QueryId, NodePropertyQuery] = field(default_factory=dict)

    @staticmethod
    def deserialize(bytes_: bytes) -> NodePropertyQueryMap:
        msg = NodePropertyQueryMapProto()
        return NodePropertyQueryMap.from_proto(msg.FromString(bytes_))

    @staticmethod
    def from_proto(proto: NodePropertyQueryMapProto) -> NodePropertyQueryMap:
        return NodePropertyQueryMap(
            entries={
                QueryId.from_proto(entry.query_id): NodePropertyQuery.from_proto(
                    entry.node_property_query
                )
                for entry in proto.entries
            }
        )

    def into_proto(self) -> NodePropertyQueryMapProto:
        msg = NodePropertyQueryMapProto()

        msg.entries.extend(
            (
                NodePropertyQueryEntryProto(
                    query_id=s.into_proto(), node_property_query=d.into_proto()
                )
                for s, d in self.entries.items()
            )
        )

        return msg

    def __getitem__(self, key: QueryId) -> NodePropertyQuery:
        return self.entries[key]

    def __setitem__(self, key: QueryId, value: NodePropertyQuery) -> None:
        self.entries[key] = value


@dataclass(frozen=True, slots=True)
class GraphQuery(SerDe[GraphQueryProto]):
    root_query_id: QueryId
    node_property_queries: NodePropertyQueryMap = field(
        default_factory=NodePropertyQueryMap
    )
    edge_filters: EdgeQueryMap = field(default_factory=EdgeQueryMap)
    edge_map: EdgeMap = field(default_factory=EdgeMap)

    @staticmethod
    def deserialize(bytes_: bytes) -> GraphQuery:
        msg = GraphQueryProto()
        return GraphQuery.from_proto(msg.FromString(bytes_))

    @staticmethod
    def from_proto(proto: GraphQueryProto) -> GraphQuery:
        return GraphQuery(
            root_query_id=QueryId.from_proto(proto.root_query_id),
            node_property_queries=NodePropertyQueryMap.from_proto(
                proto.node_property_queries
            ),
            edge_filters=EdgeQueryMap.from_proto(proto.edge_filters),
            edge_map=EdgeMap.from_proto(proto.edge_map),
        )

    def into_proto(self) -> GraphQueryProto:
        msg = GraphQueryProto()
        msg.root_query_id.CopyFrom(self.root_query_id.into_proto())
        msg.node_property_queries.CopyFrom(self.node_property_queries.into_proto())
        msg.edge_filters.CopyFrom(self.edge_filters.into_proto())
        msg.edge_map.CopyFrom(self.edge_map.into_proto())

        return msg

    @staticmethod
    def from_node_query(root_node_query: NodeQuery) -> GraphQuery:
        node_property_queries: NodePropertyQueryMap = NodePropertyQueryMap()
        edge_filters: EdgeQueryMap = EdgeQueryMap()
        edge_map: EdgeMap = EdgeMap()

        node_property_queries[
            root_node_query.node_property_query.query_id
        ] = NodePropertyQuery.from_node_query(root_node_query)
        for node, edge, neighbor in NodeQueryIterator(root_node_query):
            node_property_queries[
                node.node_property_query.query_id
            ] = NodePropertyQuery.from_node_query(node)
            edge_filters[(node.node_property_query.query_id, edge)].add(
                neighbor.node_property_query.query_id
            )
            edge_map.update(node.edge_map)

        return GraphQuery(
            root_query_id=root_node_query.node_property_query.query_id,
            node_property_queries=node_property_queries,
            edge_filters=edge_filters,
            edge_map=edge_map,
        )

    def query_with_uid(
        self,
        uid: Uid,
        client: GraphQueryClient,
    ) -> Optional[NodeView]:
        response = client.query_with_uid(
            node_uid=uid,
            graph_query=self,
        )
        if not response:
            return None

        self_node = response.nodes.get(uid)
        if not self_node:
            raise Exception("Graph did not contain 'self'")

        return NodeView.from_parts(
            node_properties=self_node,
            graph=response,
            graph_query_client=client,
        )


@dataclass(frozen=True, slots=True)
class StringProperties(SerDe[StringPropertiesProto]):
    properties: Dict[PropertyName, str] = field(default_factory=dict)

    @staticmethod
    def deserialize(bytes_: bytes) -> StringProperties:
        msg = StringPropertiesProto()
        return StringProperties.from_proto(msg.FromString(bytes_))

    @staticmethod
    def from_proto(proto: StringPropertiesProto) -> StringProperties:
        return StringProperties(
            properties={
                PropertyName.from_proto(entry.property_name): entry.property_value
                for entry in proto.properties
            }
        )

    def into_proto(self) -> StringPropertiesProto:
        msg = StringPropertiesProto()

        msg.properties.extend(
            (
                StringPropertyProto(property_name=n.into_proto(), property_value=v)
                for n, v in self.properties.items()
            )
        )

        return msg

    def update(self, properties: StringProperties) -> None:
        self.properties.update(properties.properties)


@dataclass(frozen=True, slots=True)
class NodePropertiesView(SerDe[NodePropertiesViewProto]):
    uid: Uid
    node_type: NodeType
    string_properties: StringProperties

    # int_properties: IntProperties

    @staticmethod
    def deserialize(bytes_: bytes) -> NodePropertiesView:
        msg = NodePropertiesViewProto()
        return NodePropertiesView.from_proto(msg.FromString(bytes_))

    @staticmethod
    def from_proto(proto: NodePropertiesViewProto) -> NodePropertiesView:
        return NodePropertiesView(
            uid=Uid.from_proto(proto.uid),
            node_type=NodeType.from_proto(proto.node_type),
            string_properties=StringProperties.from_proto(proto.string_properties),
        )

    def into_proto(self) -> NodePropertiesViewProto:
        msg = NodePropertiesViewProto()

        msg.uid.CopyFrom(self.uid.into_proto())
        msg.node_type.CopyFrom(self.node_type.into_proto())
        msg.string_properties.CopyFrom(self.string_properties.into_proto())
        # msg.int_properties.CopyFrom(self.int_properties.into_proto())

        return msg

    def merge(self, other: NodePropertiesView) -> None:
        if self.uid != other.uid:
            raise Exception("Attempted to merge two nodes with different uid")
        if self.node_type != other.node_type:
            raise Exception("Attempted to merge two nodes with different node types")

        self.string_properties.update(other.string_properties)


@dataclass(frozen=True, slots=True)
class NodePropertiesViewMap(SerDe[NodePropertiesViewMapProto]):
    entries: Dict[Uid, NodePropertiesView] = field(default_factory=dict)

    def property_views(self) -> Iterator[NodePropertiesView]:
        return iter(self.entries.values())

    @staticmethod
    def deserialize(bytes_: bytes) -> NodePropertiesViewMap:
        msg = NodePropertiesViewMapProto()
        return NodePropertiesViewMap.from_proto(msg.FromString(bytes_))

    @staticmethod
    def from_proto(proto: NodePropertiesViewMapProto) -> NodePropertiesViewMap:
        return NodePropertiesViewMap(
            entries={
                Uid.from_proto(entry.uid): NodePropertiesView.from_proto(
                    entry.node_view
                )
                for entry in proto.entries
            }
        )

    def into_proto(self) -> NodePropertiesViewMapProto:
        msg = NodePropertiesViewMapProto()
        msg.entries.extend(
            (
                NodePropertiesViewEntryProto(
                    uid=s.into_proto(), node_view=d.into_proto()
                )
                for s, d in self.entries.items()
            )
        )
        return msg

    def __getitem__(self, key: Uid) -> NodePropertiesView:
        return self.entries[key]

    def get(self, key: Uid) -> Optional[NodePropertiesView]:
        return self.entries.get(key)

    def update(self, other: NodePropertiesViewMap) -> None:
        for other_uid, entry in other.entries.items():
            if existing_node := self.get(other_uid):
                existing_node.merge(entry)
            else:
                self.entries[other_uid] = entry


@dataclass(frozen=True, slots=True)
class EdgeViewMap(SerDe[EdgeViewMapProto]):
    entries: Dict[Tuple[Uid, EdgeName], Set[Uid]] = field(default_factory=dict)

    @staticmethod
    def deserialize(bytes_: bytes) -> EdgeViewMap:
        msg = EdgeViewMapProto()
        return EdgeViewMap.from_proto(msg.FromString(bytes_))

    @staticmethod
    def from_proto(proto: EdgeViewMapProto) -> EdgeViewMap:
        return EdgeViewMap(
            entries={
                (Uid.from_proto(entry.uid), EdgeName.from_proto(entry.edge_name)): set(
                    (Uid.from_proto(n) for n in entry.neighbors)
                )
                for entry in proto.entries
            }
        )

    def into_proto(self) -> EdgeViewMapProto:
        msg = EdgeViewMapProto()

        msg.entries.extend(
            (
                EdgeViewEntryProto(
                    uid=uid.into_proto(),
                    edge_name=edge_name.into_proto(),
                    neighbors=(n.into_proto() for n in neighbors),
                )
                for ((uid, edge_name), neighbors) in self.entries.items()
            )
        )

        return msg

    def __getitem__(self, key: Tuple[Uid, EdgeName]) -> Set[Uid]:
        return self.entries[key]

    def get(self, key: Tuple[Uid, EdgeName]) -> Optional[Set[Uid]]:
        return self.entries.get(key)


@dataclass(frozen=True, slots=True)
class GraphView(SerDe[GraphViewProto]):
    nodes: NodePropertiesViewMap = field(default_factory=NodePropertiesViewMap)
    edges: EdgeViewMap = field(default_factory=EdgeViewMap)

    @staticmethod
    def deserialize(bytes_: bytes) -> GraphView:
        msg = GraphViewProto()
        return GraphView.from_proto(msg.FromString(bytes_))

    @staticmethod
    def from_proto(proto: GraphViewProto) -> GraphView:
        return GraphView(
            nodes=NodePropertiesViewMap.from_proto(proto.nodes),
            edges=EdgeViewMap.from_proto(proto.edges),
        )

    def into_proto(self) -> GraphViewProto:
        msg = GraphViewProto()
        msg.nodes.CopyFrom(self.nodes.into_proto())
        msg.edges.CopyFrom(self.edges.into_proto())
        return msg

    def get_node(self, uid: Uid) -> NodePropertiesView | None:
        return self.nodes.get(uid)

    def merge(self, other: GraphView) -> None:
        self.nodes.update(other.nodes)


@dataclass(frozen=True, slots=True)
class QueryGraphWithNodeRequest(SerDe[QueryGraphWithNodeRequestProto]):
    tenant_id: Uuid
    node_uid: Uid
    graph_query: GraphQuery

    @staticmethod
    def deserialize(bytes_: bytes) -> QueryGraphWithNodeRequest:
        msg = QueryGraphWithNodeRequestProto()
        return QueryGraphWithNodeRequest.from_proto(msg.FromString(bytes_))

    @staticmethod
    def from_proto(proto: QueryGraphWithNodeRequestProto) -> QueryGraphWithNodeRequest:
        return QueryGraphWithNodeRequest(
            tenant_id=Uuid.from_proto(proto.tenant_id),
            node_uid=Uid.from_proto(proto.node_uid),
            graph_query=GraphQuery.from_proto(proto.graph_query),
        )

    def into_proto(self) -> QueryGraphWithNodeRequestProto:
        msg = QueryGraphWithNodeRequestProto()
        msg.graph_query.CopyFrom(self.graph_query.into_proto())
        msg.tenant_id.CopyFrom(self.tenant_id.into_proto())
        msg.node_uid.CopyFrom(self.node_uid.into_proto())
        return msg


@dataclass(frozen=True, slots=True)
class QueryGraphWithNodeResponse:
    matched_graph: GraphView
    root_uid: Uid

    @staticmethod
    def deserialize(bytes_: bytes) -> QueryGraphWithNodeResponse:
        msg = QueryGraphWithNodeResponseProto()
        return QueryGraphWithNodeResponse.from_proto(msg.FromString(bytes_))

    @staticmethod
    def from_proto(
        proto: QueryGraphWithNodeResponseProto,
    ) -> QueryGraphWithNodeResponse:
        return QueryGraphWithNodeResponse(
            matched_graph=GraphView.from_proto(proto.matched_graph),
            root_uid=Uid.from_proto(proto.root_uid),
        )

    def into_proto(self) -> QueryGraphWithNodeResponseProto:
        msg = QueryGraphWithNodeResponseProto()
        msg.matched_graph.CopyFrom(self.matched_graph.into_proto())
        msg.root_uid.CopyFrom(self.root_uid.into_proto())
        return msg


@dataclass(frozen=True, slots=True)
class QueryGraphFromNodeRequest(SerDe[QueryGraphFromNodeRequestProto]):
    tenant_id: Uuid
    node_uid: Uid
    graph_query: GraphQuery

    @staticmethod
    def deserialize(bytes_: bytes) -> QueryGraphFromNodeRequest:
        msg = QueryGraphFromNodeRequestProto()
        return QueryGraphFromNodeRequest.from_proto(msg.FromString(bytes_))

    @staticmethod
    def from_proto(proto: QueryGraphFromNodeRequestProto) -> QueryGraphFromNodeRequest:
        return QueryGraphFromNodeRequest(
            tenant_id=Uuid.from_proto(proto.tenant_id),
            node_uid=Uid.from_proto(proto.node_uid),
            graph_query=GraphQuery.from_proto(proto.graph_query),
        )

    def into_proto(self) -> QueryGraphFromNodeRequestProto:
        msg = QueryGraphFromNodeRequestProto()
        msg.graph_query.CopyFrom(self.graph_query.into_proto())
        msg.tenant_id.CopyFrom(self.tenant_id.into_proto())
        msg.node_uid.CopyFrom(self.node_uid.into_proto())
        return msg


@dataclass(frozen=True, slots=True)
class QueryGraphFromNodeResponse:
    matched_graph: GraphView

    @staticmethod
    def deserialize(bytes_: bytes) -> QueryGraphFromNodeResponse:
        msg = QueryGraphFromNodeResponseProto()
        return QueryGraphFromNodeResponse.from_proto(msg.FromString(bytes_))

    @staticmethod
    def from_proto(
        proto: QueryGraphFromNodeResponseProto,
    ) -> QueryGraphFromNodeResponse:
        return QueryGraphFromNodeResponse(
            matched_graph=GraphView.from_proto(proto.matched_graph),
        )

    def into_proto(self) -> QueryGraphFromNodeResponseProto:
        msg = QueryGraphFromNodeResponseProto()
        msg.matched_graph.CopyFrom(self.matched_graph.into_proto())
        return msg


"""

GRPC Client Implementation

This is in the same file because Python can't handle circular imports

"""


@dataclass(frozen=True, slots=True)
class GraphQueryClient:
    tenant_id: Uuid
    client_stub: GraphQueryServiceStub

    def query_with_uid(
        self,
        node_uid: Uid,
        graph_query: GraphQuery,
    ) -> Optional[GraphView]:
        request = QueryGraphWithNodeRequest(
            tenant_id=self.tenant_id,
            node_uid=node_uid,
            graph_query=graph_query,
        ).into_proto()

        response = QueryGraphWithNodeResponse.from_proto(
            self.client_stub.QueryGraphWithUid(
                request=request,
            )
        )

        return None

    def query_from_uid(
        self,
        node_uid: Uid,
        graph_query: GraphQuery,
    ) -> Optional[GraphView]:
        request = QueryGraphFromNodeRequest(
            tenant_id=self.tenant_id,
            node_uid=node_uid,
            graph_query=graph_query,
        ).into_proto()
        response = self.client_stub.QueryGraphFromUid(request)
        if response.matched_graph:
            return QueryGraphFromNodeResponse.from_proto(response).matched_graph
        return None


"""
BELOW IS NOT RELATED TO PROTOBUF, THESE ARE HELPERS

|
|
|
|
|
|


"""


@dataclass(frozen=True, slots=True)
class EdgeFilters:
    node_queries: List[NodeQuery] = field(default_factory=list)

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
    edge_map: Dict[EdgeName, EdgeName] = field(default_factory=dict)

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


@dataclass(frozen=False)
class NodeQueryIterator:
    parent: NodeQuery
    query_ids: Dict[NodeQuery, QueryId] = field(default_factory=dict)
    visited: Set[Tuple[QueryId, EdgeName, QueryId]] = field(default_factory=set)
    neighbors: Deque[NodeQuery] = field(default_factory=deque)

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

    @staticmethod
    def from_parts(
        node_properties: NodePropertiesView,
        graph: GraphView,
        graph_query_client: GraphQueryClient,
    ) -> NodeView:
        return NodeView(
            uid=node_properties.uid,
            node_type=node_properties.node_type,
            string_properties=node_properties.string_properties,
            # int_properties=node_properties.int_properties,
            graph=graph,
            graph_query_client=graph_query_client,
        )

    def get_node(self, node_uid: Uid) -> Optional[NodeView]:
        if n := self.graph.nodes.get(node_uid):
            return NodeView.from_parts(n, self.graph, self.graph_query_client)
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

        graph_query = GraphQuery.from_node_query(node_query)

        if response := self.graph_query_client.query_from_uid(
            self.uid,
            graph_query,
        ):
            self.graph.merge(response)




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
    string_property_schemas: Dict[PropertyName, StringPropertySchema] = field(default_factory=dict)
    edge_schemas: Dict[EdgeName, EdgeSchema] = field(default_factory=dict)


def main() -> None:
    NodePropertyQueryMap()
    GraphQuery(QueryId())


if __name__ == "__main__":
    main()
