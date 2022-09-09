from __future__ import annotations

import enum
import os
from collections import defaultdict
from dataclasses import dataclass, field
from typing import DefaultDict, Iterable, Iterator, Mapping, Optional, Sequence

from graplinc.grapl.api.graph_query_service.v1beta1 import (
    graph_query_service_pb2 as proto,
)
from python_proto.common import Uuid
from python_proto.grapl.common.v1beta1.messages import (
    EdgeName,
    NodeType,
    PropertyName,
    Uid,
)
from python_proto.serde import SerDe


def _non_zero_int() -> int:
    """
    A helper function for returning a 64bit integer that is never
    zero. We use this for Query IDs.
    :return: A random 64bit integer that will never be zero
    """
    return int.from_bytes(os.urandom(8), "little") | 1


@enum.unique
class IntFilterOperation(enum.IntEnum):
    HAS = 1
    EQUAL = 2
    LESS_THAN = 3
    LESS_THAN_OR_EQUAL = 4
    GREATER_THAN = 5
    GREATER_THAN_OR_EQUAL = 6


@dataclass(frozen=True, slots=True)
class IntFilter(SerDe[proto.IntFilter]):
    operation: IntFilterOperation
    value: int
    negated: bool
    _proto_cls = proto.IntFilter

    @classmethod
    def from_proto(cls, proto_value: proto.IntFilter) -> IntFilter:
        return cls(
            operation=IntFilterOperation(proto_value.operation),
            value=proto_value.value,
            negated=proto_value.negated,
        )

    def into_proto(self) -> proto.IntFilter:
        msg = self.new_proto()
        # type ignored because pants' mypy-protobuf is not up-to-date
        msg.operation = self.operation.value  # type: ignore
        msg.value = self.value
        msg.negated = self.negated
        return msg


@dataclass(frozen=True, slots=True)
class AndIntFilters(SerDe[proto.AndIntFilters]):
    int_filters: list[IntFilter] = field(default_factory=list)
    _proto_cls = proto.AndIntFilters

    @classmethod
    def from_proto(cls, proto_value: proto.AndIntFilters) -> AndIntFilters:
        return cls(
            int_filters=[IntFilter.from_proto(p) for p in proto_value.int_filters]
        )

    def into_proto(self) -> proto.AndIntFilters:
        msg = self.new_proto()
        msg.int_filters.extend(p.into_proto() for p in self.int_filters)
        return msg

    def __len__(self) -> int:
        return len(self.int_filters)

    def __iter__(self) -> Iterator[IntFilter]:
        return self.int_filters.__iter__()


@dataclass(frozen=True, slots=True)
class OrIntFilters(SerDe[proto.OrIntFilters]):
    and_int_filters: list[AndIntFilters] = field(default_factory=list)
    _proto_cls = proto.OrIntFilters

    @classmethod
    def from_proto(cls, proto_value: proto.OrIntFilters) -> OrIntFilters:
        return cls(
            and_int_filters=[
                AndIntFilters.from_proto(p) for p in proto_value.and_int_filters
            ]
        )

    def into_proto(self) -> proto.OrIntFilters:
        msg = self.new_proto()
        msg.and_int_filters.extend(p.into_proto() for p in self.and_int_filters)
        return msg

    def append(self, and_int_filters: AndIntFilters) -> None:
        self.and_int_filters.append(and_int_filters)

    def __len__(self) -> int:
        return len(self.and_int_filters)

    def __iter__(self) -> Iterator[AndIntFilters]:
        return self.and_int_filters.__iter__()


@dataclass(frozen=True, slots=True)
class QueryId(SerDe[proto.QueryId]):
    value: int = field(default_factory=_non_zero_int)
    _proto_cls = proto.QueryId

    @classmethod
    def from_proto(cls, proto_value: proto.QueryId) -> QueryId:
        return QueryId(
            value=proto_value.value,
        )

    def into_proto(self) -> proto.QueryId:
        msg = proto.QueryId()
        msg.value = self.value

        return msg

    def __hash__(self) -> int:
        return self.value


@enum.unique
class StringOperation(enum.IntEnum):
    HAS = 1
    EQUAL = 2
    CONTAINS = 3
    REGEX = 4


@dataclass(frozen=True, slots=True)
class StringFilter(SerDe[proto.StringFilter]):
    operation: StringOperation
    value: str
    negated: bool
    _proto_cls = proto.StringFilter

    @classmethod
    def from_proto(cls, proto: proto.StringFilter) -> StringFilter:
        return StringFilter(
            operation=StringOperation(proto.operation),
            value=proto.value,
            negated=proto.negated,
        )

    def into_proto(self) -> proto.StringFilter:
        msg = proto.StringFilter()
        # type ignored because pants' mypy-protobuf is not up-to-date
        msg.operation = self.operation.value  # type: ignore
        msg.value = self.value
        msg.negated = self.negated
        return msg


@dataclass(frozen=True, slots=True)
class AndStringFilters(SerDe[proto.AndStringFilters]):
    string_filters: list[StringFilter] = field(default_factory=list)
    _proto_cls = proto.AndStringFilters

    @classmethod
    def from_proto(cls, proto: proto.AndStringFilters) -> AndStringFilters:
        return AndStringFilters(
            string_filters=[StringFilter.from_proto(p) for p in proto.string_filters]
        )

    def into_proto(self) -> proto.AndStringFilters:
        msg = proto.AndStringFilters()
        msg.string_filters.extend(p.into_proto() for p in self.string_filters)
        return msg

    def __len__(self) -> int:
        return len(self.string_filters)

    def __iter__(self) -> Iterator[StringFilter]:
        return self.string_filters.__iter__()


@dataclass(frozen=True, slots=True)
class OrStringFilters(SerDe[proto.OrStringFilters]):
    and_string_filters: list[AndStringFilters] = field(default_factory=list)
    _proto_cls = proto.OrStringFilters

    @classmethod
    def from_proto(cls, proto: proto.OrStringFilters) -> OrStringFilters:
        return OrStringFilters(
            and_string_filters=[
                AndStringFilters.from_proto(p) for p in proto.and_string_filters
            ]
        )

    def into_proto(self) -> proto.OrStringFilters:
        msg = proto.OrStringFilters()
        msg.and_string_filters.extend(p.into_proto() for p in self.and_string_filters)
        return msg

    def append(self, and_string_filters: AndStringFilters) -> None:
        self.and_string_filters.append(and_string_filters)

    def __len__(self) -> int:
        return len(self.and_string_filters)

    def __iter__(self) -> Iterator[AndStringFilters]:
        return self.and_string_filters.__iter__()


@dataclass(frozen=True, slots=True)
class EdgeQueryEntry(SerDe[proto.EdgeQueryEntry]):
    edge_name: EdgeName
    neighbor_query_ids: set[QueryId]
    query_id: QueryId = field(default_factory=QueryId)
    _proto_cls = proto.EdgeQueryEntry

    @classmethod
    def from_proto(cls, proto: proto.EdgeQueryEntry) -> EdgeQueryEntry:
        return EdgeQueryEntry(
            query_id=QueryId.from_proto(proto.query_id),
            edge_name=EdgeName.from_proto(proto.edge_name),
            neighbor_query_ids={
                QueryId.from_proto(p) for p in proto.neighbor_query_ids
            },
        )

    def into_proto(self) -> proto.EdgeQueryEntry:
        msg = proto.EdgeQueryEntry()
        msg.query_id.CopyFrom(self.query_id.into_proto())
        msg.edge_name.CopyFrom(self.edge_name.into_proto())
        msg.neighbor_query_ids.extend(p.into_proto() for p in self.neighbor_query_ids)
        return msg


@enum.unique
class UidOperation(enum.IntEnum):
    EQUAL = 1


@dataclass(frozen=True, slots=True)
class UidFilter(SerDe[proto.UidFilter]):
    operation: UidOperation
    value: Uid
    _proto_cls = proto.UidFilter

    @classmethod
    def from_proto(cls, proto: proto.UidFilter) -> UidFilter:
        return UidFilter(
            operation=UidOperation(proto.operation),
            value=Uid.from_proto(proto.value),
        )

    def into_proto(self) -> proto.UidFilter:
        msg = proto.UidFilter()
        # type ignored because pants' mypy-protobuf is not up-to-date
        msg.operation = self.operation.value  # type: ignore
        msg.value.CopyFrom(self.value.into_proto())
        return msg


@dataclass(frozen=True, slots=True)
class UidFilters(SerDe[proto.UidFilters]):
    uid_filters: list[UidFilter] = field(default_factory=list)
    _proto_cls = proto.UidFilters

    @classmethod
    def from_proto(cls, proto: proto.UidFilters) -> UidFilters:
        return cls(
            uid_filters=[UidFilter.from_proto(uf) for uf in proto.uid_filters],
        )

    def into_proto(self) -> proto.UidFilters:
        msg = self.new_proto()
        msg.uid_filters.extend([uf.into_proto() for uf in self.uid_filters])
        return msg

    def extend(self, iterable: Iterable[UidFilter]) -> None:
        self.uid_filters.extend(iterable)


EdgeQueryMapK = tuple[QueryId, EdgeName]
EdgeQueryMapV = set[QueryId]


@dataclass(frozen=True, slots=True)
class EdgeQueryMap(SerDe[proto.EdgeQueryMap]):
    entries: dict[EdgeQueryMapK, EdgeQueryMapV] = field(default_factory=dict)
    _proto_cls = proto.EdgeQueryMap

    @classmethod
    def from_proto(cls, proto: proto.EdgeQueryMap) -> EdgeQueryMap:
        return EdgeQueryMap(
            entries={
                (
                    QueryId.from_proto(entry.query_id),
                    EdgeName.from_proto(entry.edge_name),
                ): {QueryId.from_proto(n) for n in entry.neighbor_query_ids}
                for entry in proto.entries
            }
        )

    def into_proto(self) -> proto.EdgeQueryMap:
        msg = proto.EdgeQueryMap()

        entries = (
            EdgeQueryEntry(
                query_id=query_id, edge_name=edge_name, neighbor_query_ids=neighbors
            ).into_proto()
            for ((query_id, edge_name), neighbors) in self.entries.items()
        )

        msg.entries.extend(entries)
        return msg

    def __getitem__(self, key: tuple[QueryId, EdgeName]) -> set[QueryId]:
        return self.entries[key]

    def __setitem__(self, key: tuple[QueryId, EdgeName], value: set[QueryId]) -> None:
        self.entries[key] = value


@dataclass(frozen=True, slots=True)
class NodePropertyQuery(SerDe[proto.NodePropertyQuery]):
    node_type: NodeType
    query_id: QueryId = field(default_factory=QueryId)
    string_filters: DefaultDict[PropertyName, OrStringFilters] = field(
        default_factory=lambda: defaultdict(OrStringFilters)
    )
    int_filters: DefaultDict[PropertyName, OrIntFilters] = field(
        default_factory=lambda: defaultdict(OrIntFilters)
    )
    uid_filters: UidFilters = field(default_factory=UidFilters)
    _proto_cls = proto.NodePropertyQuery

    @classmethod
    def from_proto(cls, proto: proto.NodePropertyQuery) -> NodePropertyQuery:
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
            int_filters=defaultdict(
                OrIntFilters,
                {
                    PropertyName(p): OrIntFilters.from_proto(f)
                    for p, f in proto.int_filters.items()
                },
            ),
            uid_filters=UidFilters.from_proto(proto.uid_filters),
        )

    def into_proto(self) -> proto.NodePropertyQuery:
        msg = proto.NodePropertyQuery()
        msg.query_id.CopyFrom(self.query_id.into_proto())
        msg.node_type.CopyFrom(self.node_type.into_proto())
        for p, string_filter in self.string_filters.items():
            msg.string_filters[p.value].CopyFrom(string_filter.into_proto())
        for p, int_filter in self.int_filters.items():
            msg.int_filters[p.value].CopyFrom(int_filter.into_proto())
        msg.uid_filters.CopyFrom(self.uid_filters.into_proto())
        return msg

    def with_string_filters(
        self, property_name: PropertyName, filters: Sequence[StringFilter]
    ) -> NodePropertyQuery:
        self.string_filters[property_name].append(AndStringFilters(list(filters)))
        return self

    def with_uid_filters(self, filters: Sequence[UidFilter]) -> NodePropertyQuery:
        self.uid_filters.extend(filters)
        return self


@dataclass(frozen=True, slots=True)
class EdgeNameMap(SerDe[proto.EdgeNameMap]):
    entries: dict[EdgeName, EdgeName] = field(default_factory=dict)

    _proto_cls = proto.EdgeNameMap

    @classmethod
    def from_proto(cls, proto: proto.EdgeNameMap) -> EdgeNameMap:
        return EdgeNameMap(
            entries={
                EdgeName.from_proto(entry.forward_edge_name): EdgeName.from_proto(
                    entry.reverse_edge_name
                )
                for entry in proto.entries
            }
        )

    def into_proto(self) -> proto.EdgeNameMap:
        msg = self.new_proto()
        msg.entries.extend(
            (
                proto.EdgeNameEntry(
                    forward_edge_name=s.into_proto(), reverse_edge_name=d.into_proto()
                )
                for s, d in self.entries.items()
            )
        )

        return msg

    @classmethod
    def from_entry(
        cls, src_edge_name: EdgeName, dst_edge_name: EdgeName
    ) -> EdgeNameMap:
        return cls(
            entries={
                src_edge_name: dst_edge_name,
                dst_edge_name: src_edge_name,
            }
        )

    def update(self, other: Mapping[EdgeName, EdgeName]) -> None:
        self.entries.update(other)

    def with_mapping(self, src_name: EdgeName, dst_name: EdgeName) -> EdgeNameMap:
        self.entries[src_name] = dst_name
        self.entries[dst_name] = src_name
        return self


@dataclass(frozen=True, slots=True)
class NodePropertyQueryMap(SerDe[proto.NodePropertyQueryMap]):
    entries: dict[QueryId, NodePropertyQuery] = field(default_factory=dict)
    _proto_cls = proto.NodePropertyQueryMap

    @classmethod
    def from_proto(cls, proto: proto.NodePropertyQueryMap) -> NodePropertyQueryMap:
        return NodePropertyQueryMap(
            entries={
                QueryId.from_proto(entry.query_id): NodePropertyQuery.from_proto(
                    entry.node_property_query
                )
                for entry in proto.entries
            }
        )

    def into_proto(self) -> proto.NodePropertyQueryMap:
        msg = proto.NodePropertyQueryMap()

        msg.entries.extend(
            (
                proto.NodePropertyQueryEntry(
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
class GraphQuery(SerDe[proto.GraphQuery]):
    root_query_id: QueryId
    node_property_queries: NodePropertyQueryMap = field(
        default_factory=NodePropertyQueryMap
    )
    edge_filters: EdgeQueryMap = field(default_factory=EdgeQueryMap)
    edge_map: EdgeNameMap = field(default_factory=EdgeNameMap)
    _proto_cls = proto.GraphQuery

    @classmethod
    def from_proto(cls, proto: proto.GraphQuery) -> GraphQuery:
        return GraphQuery(
            root_query_id=QueryId.from_proto(proto.root_query_id),
            node_property_queries=NodePropertyQueryMap.from_proto(
                proto.node_property_queries
            ),
            edge_filters=EdgeQueryMap.from_proto(proto.edge_filters),
            edge_map=EdgeNameMap.from_proto(proto.edge_map),
        )

    def into_proto(self) -> proto.GraphQuery:
        msg = proto.GraphQuery()
        msg.root_query_id.CopyFrom(self.root_query_id.into_proto())
        msg.node_property_queries.CopyFrom(self.node_property_queries.into_proto())
        msg.edge_filters.CopyFrom(self.edge_filters.into_proto())
        msg.edge_map.CopyFrom(self.edge_map.into_proto())

        return msg


@dataclass(frozen=True, slots=True)
class StringProperties(SerDe[proto.StringProperties]):
    properties: dict[PropertyName, str] = field(default_factory=dict)
    _proto_cls = proto.StringProperties

    @classmethod
    def from_proto(cls, proto: proto.StringProperties) -> StringProperties:
        return StringProperties(
            properties={
                PropertyName.from_proto(entry.property_name): entry.property_value
                for entry in proto.properties
            }
        )

    def into_proto(self) -> proto.StringProperties:
        msg = proto.StringProperties()

        msg.properties.extend(
            (
                proto.StringProperty(property_name=n.into_proto(), property_value=v)
                for n, v in self.properties.items()
            )
        )

        return msg

    def update(self, properties: StringProperties) -> None:
        self.properties.update(properties.properties)


@dataclass(frozen=True, slots=True)
class NodePropertiesView(SerDe[proto.NodePropertiesView]):
    uid: Uid
    node_type: NodeType
    string_properties: StringProperties
    _proto_cls = proto.NodePropertiesView

    @classmethod
    def from_proto(cls, proto: proto.NodePropertiesView) -> NodePropertiesView:
        return NodePropertiesView(
            uid=Uid.from_proto(proto.uid),
            node_type=NodeType.from_proto(proto.node_type),
            string_properties=StringProperties.from_proto(proto.string_properties),
        )

    def into_proto(self) -> proto.NodePropertiesView:
        msg = proto.NodePropertiesView()

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
class NodePropertiesViewMap(SerDe[proto.NodePropertiesViewMap]):
    entries: dict[Uid, NodePropertiesView] = field(default_factory=dict)
    _proto_cls = proto.NodePropertiesViewMap

    def property_views(self) -> Iterator[NodePropertiesView]:
        return iter(self.entries.values())

    @classmethod
    def from_proto(cls, proto: proto.NodePropertiesViewMap) -> NodePropertiesViewMap:
        return NodePropertiesViewMap(
            entries={
                Uid.from_proto(entry.uid): NodePropertiesView.from_proto(
                    entry.node_view
                )
                for entry in proto.entries
            }
        )

    def into_proto(self) -> proto.NodePropertiesViewMap:
        msg = proto.NodePropertiesViewMap()
        msg.entries.extend(
            (
                proto.NodePropertiesViewEntry(
                    uid=s.into_proto(), node_view=d.into_proto()
                )
                for s, d in self.entries.items()
            )
        )
        return msg

    def __getitem__(self, key: Uid) -> NodePropertiesView:
        return self.entries[key]

    def get(self, key: Uid) -> NodePropertiesView | None:
        return self.entries.get(key)

    def update(self, other: NodePropertiesViewMap) -> None:
        for other_uid, entry in other.entries.items():
            if existing_node := self.get(other_uid):
                existing_node.merge(entry)
            else:
                self.entries[other_uid] = entry


EdgeViewMapK = tuple[Uid, EdgeName]


@dataclass(frozen=True, slots=True)
class EdgeViewMap(SerDe[proto.EdgeViewMap]):
    entries: dict[EdgeViewMapK, set[Uid]] = field(default_factory=dict)
    _proto_cls = proto.EdgeViewMap

    @classmethod
    def from_proto(cls, proto: proto.EdgeViewMap) -> EdgeViewMap:
        return EdgeViewMap(
            entries={
                (Uid.from_proto(entry.uid), EdgeName.from_proto(entry.edge_name)): {
                    Uid.from_proto(n) for n in entry.neighbors
                }
                for entry in proto.entries
            }
        )

    def into_proto(self) -> proto.EdgeViewMap:
        msg = proto.EdgeViewMap()

        msg.entries.extend(
            proto.EdgeViewEntry(
                uid=uid.into_proto(),
                edge_name=edge_name.into_proto(),
                neighbors=(n.into_proto() for n in neighbors),
            )
            for ((uid, edge_name), neighbors) in self.entries.items()
        )

        return msg

    def __getitem__(self, key: tuple[Uid, EdgeName]) -> set[Uid]:
        return self.entries[key]

    def get(self, key: tuple[Uid, EdgeName]) -> set[Uid] | None:
        return self.entries.get(key)


@dataclass(frozen=True, slots=True)
class GraphView(SerDe[proto.GraphView]):
    nodes: NodePropertiesViewMap = field(default_factory=NodePropertiesViewMap)
    edges: EdgeViewMap = field(default_factory=EdgeViewMap)
    _proto_cls = proto.GraphView

    @classmethod
    def from_proto(cls, proto: proto.GraphView) -> GraphView:
        return GraphView(
            nodes=NodePropertiesViewMap.from_proto(proto.nodes),
            edges=EdgeViewMap.from_proto(proto.edges),
        )

    def into_proto(self) -> proto.GraphView:
        msg = proto.GraphView()
        msg.nodes.CopyFrom(self.nodes.into_proto())
        msg.edges.CopyFrom(self.edges.into_proto())
        return msg

    def get_node(self, uid: Uid) -> NodePropertiesView | None:
        return self.nodes.get(uid)

    def merge(self, other: GraphView) -> None:
        self.nodes.update(other.nodes)


@dataclass(frozen=True, slots=True)
class MatchedGraphWithUid(SerDe[proto.MatchedGraphWithUid]):
    matched_graph: GraphView
    root_uid: Uid

    _proto_cls = proto.MatchedGraphWithUid

    @classmethod
    def from_proto(cls, proto_value: proto.MatchedGraphWithUid) -> MatchedGraphWithUid:
        return cls(
            matched_graph=GraphView.from_proto(proto_value.matched_graph),
            root_uid=Uid.from_proto(proto_value.root_uid),
        )

    def into_proto(self) -> proto.MatchedGraphWithUid:
        msg = self.new_proto()
        msg.matched_graph.CopyFrom(self.matched_graph.into_proto())
        msg.root_uid.CopyFrom(self.root_uid.into_proto())
        return msg


@dataclass(frozen=True, slots=True)
class NoMatchWithUid(SerDe[proto.NoMatchWithUid]):
    _proto_cls = proto.NoMatchWithUid

    @classmethod
    def from_proto(cls, proto_value: proto.NoMatchWithUid) -> NoMatchWithUid:
        return cls()

    def into_proto(self) -> proto.NoMatchWithUid:
        msg = self.new_proto()
        return msg


@dataclass(frozen=True, slots=True)
class QueryGraphWithUidRequest(SerDe[proto.QueryGraphWithUidRequest]):
    tenant_id: Uuid
    node_uid: Uid
    graph_query: GraphQuery
    _proto_cls = proto.QueryGraphWithUidRequest

    @classmethod
    def from_proto(
        cls, proto: proto.QueryGraphWithUidRequest
    ) -> QueryGraphWithUidRequest:
        return QueryGraphWithUidRequest(
            tenant_id=Uuid.from_proto(proto.tenant_id),
            node_uid=Uid.from_proto(proto.node_uid),
            graph_query=GraphQuery.from_proto(proto.graph_query),
        )

    def into_proto(self) -> proto.QueryGraphWithUidRequest:
        msg = proto.QueryGraphWithUidRequest()
        msg.graph_query.CopyFrom(self.graph_query.into_proto())
        msg.tenant_id.CopyFrom(self.tenant_id.into_proto())
        msg.node_uid.CopyFrom(self.node_uid.into_proto())
        return msg


@dataclass(frozen=True, slots=True)
class QueryGraphWithUidResponse(SerDe[proto.QueryGraphWithUidResponse]):
    maybe_match: MaybeMatchWithUid
    _proto_cls = proto.QueryGraphWithUidResponse

    @classmethod
    def from_proto(
        cls,
        proto: proto.QueryGraphWithUidResponse,
    ) -> QueryGraphWithUidResponse:
        return cls(
            maybe_match=MaybeMatchWithUid.from_proto(proto.maybe_match),
        )

    def into_proto(self) -> proto.QueryGraphWithUidResponse:
        msg = self.new_proto()
        msg.maybe_match.CopyFrom(self.maybe_match.into_proto())
        return msg


@dataclass(frozen=True, slots=True)
class QueryGraphFromUidRequest(SerDe[proto.QueryGraphFromUidRequest]):
    tenant_id: Uuid
    node_uid: Uid
    graph_query: GraphQuery
    _proto_cls = proto.QueryGraphFromUidRequest

    @classmethod
    def from_proto(
        cls, proto: proto.QueryGraphFromUidRequest
    ) -> QueryGraphFromUidRequest:
        return cls(
            tenant_id=Uuid.from_proto(proto.tenant_id),
            node_uid=Uid.from_proto(proto.node_uid),
            graph_query=GraphQuery.from_proto(proto.graph_query),
        )

    def into_proto(self) -> proto.QueryGraphFromUidRequest:
        msg = proto.QueryGraphFromUidRequest()
        msg.graph_query.CopyFrom(self.graph_query.into_proto())
        msg.tenant_id.CopyFrom(self.tenant_id.into_proto())
        msg.node_uid.CopyFrom(self.node_uid.into_proto())
        return msg


MaybeMatchWithUidInner = MatchedGraphWithUid | NoMatchWithUid


@dataclass(frozen=True, slots=True)
class MaybeMatchWithUid(SerDe[proto.MaybeMatchWithUid]):
    inner: MaybeMatchWithUidInner
    _proto_cls = proto.MaybeMatchWithUid

    @classmethod
    def from_proto(
        cls,
        proto_value: proto.MaybeMatchWithUid,
    ) -> MaybeMatchWithUid:
        field_name = proto_value.WhichOneof("inner")
        assert field_name is not None

        match field_name:
            case "matched":
                return cls(inner=MatchedGraphWithUid.from_proto(proto_value.matched))
            case "missed":
                return cls(inner=NoMatchWithUid.from_proto(proto_value.missed))

    def into_proto(self) -> proto.MaybeMatchWithUid:
        msg = self.new_proto()
        match self.inner:
            case MatchedGraphWithUid() as inner:
                msg.matched.CopyFrom(inner.into_proto())
            case NoMatchWithUid() as inner:
                msg.missed.CopyFrom(inner.into_proto())
        return msg

    def as_optional(self) -> Optional[MatchedGraphWithUid]:
        match self.inner:
            case MatchedGraphWithUid() as inner:
                return inner
            case NoMatchWithUid() as inner:
                return None


@dataclass(frozen=True, slots=True)
class QueryGraphFromUidResponse(SerDe[proto.QueryGraphFromUidResponse]):
    matched_graph: GraphView
    _proto_cls = proto.QueryGraphFromUidResponse

    @classmethod
    def from_proto(
        cls,
        proto: proto.QueryGraphFromUidResponse,
    ) -> QueryGraphFromUidResponse:
        return QueryGraphFromUidResponse(
            matched_graph=GraphView.from_proto(proto.matched_graph),
        )

    def into_proto(self) -> proto.QueryGraphFromUidResponse:
        msg = proto.QueryGraphFromUidResponse()
        msg.matched_graph.CopyFrom(self.matched_graph.into_proto())
        return msg
