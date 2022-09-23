from __future__ import annotations

import dataclasses
from typing import Mapping, Sequence, cast

from graplinc.grapl.api.graph.v1beta1 import types_pb2 as proto
from python_proto.grapl.common.v1beta1.messages import Uid
from python_proto.serde import SerDe


@dataclasses.dataclass(frozen=True, slots=True,)
class Session(SerDe[proto.Session]):
    primary_key_properties: Sequence[str]
    primary_key_requires_asset_id: bool
    create_time: int
    last_seen_time: int
    terminate_time: int
    _proto_cls = proto.Session

    @classmethod
    def from_proto(cls, proto_session: proto.Session) -> Session:
        return Session(
            primary_key_properties=proto_session.primary_key_properties,
            primary_key_requires_asset_id=proto_session.primary_key_requires_asset_id,
            create_time=proto_session.create_time,
            last_seen_time=proto_session.last_seen_time,
            terminate_time=proto_session.terminate_time,
        )

    def into_proto(self) -> proto.Session:
        proto_session = proto.Session()
        for prop in self.primary_key_properties:
            proto_session.primary_key_properties.append(prop)
        proto_session.primary_key_requires_asset_id = self.primary_key_requires_asset_id
        proto_session.create_time = self.create_time
        proto_session.last_seen_time = self.last_seen_time
        proto_session.terminate_time = self.terminate_time
        return proto_session


@dataclasses.dataclass(frozen=True, slots=True,)
class Static(SerDe[proto.Static]):
    primary_key_properties: Sequence[str]
    primary_key_requires_asset_id: bool
    _proto_cls = proto.Static

    @classmethod
    def from_proto(cls, proto_static: proto.Static) -> Static:
        return Static(
            primary_key_properties=proto_static.primary_key_properties,
            primary_key_requires_asset_id=proto_static.primary_key_requires_asset_id,
        )

    def into_proto(self) -> proto.Static:
        proto_static = proto.Static()
        for prop in self.primary_key_properties:
            proto_static.primary_key_properties.append(prop)
        proto_static.primary_key_requires_asset_id = self.primary_key_requires_asset_id
        return proto_static


@dataclasses.dataclass(frozen=True, slots=True,)
class IdStrategy(SerDe[proto.IdStrategy]):
    strategy: Session | Static
    _proto_cls = proto.IdStrategy

    @classmethod
    def from_proto(cls, proto_id_strategy: proto.IdStrategy) -> IdStrategy:
        if proto_id_strategy.HasField("session"):
            return IdStrategy(strategy=Session.from_proto(proto_id_strategy.session))
        elif proto_id_strategy.HasField("static"):
            return IdStrategy(strategy=Static.from_proto(proto_id_strategy.static))
        else:
            raise Exception("Encountered unknown type")

    def into_proto(self) -> proto.IdStrategy:
        proto_id_strategy = proto.IdStrategy()
        if type(self.strategy) is Session:
            proto_id_strategy.session.CopyFrom(
                cast(proto.Session, self.strategy.into_proto())
            )
        elif type(self.strategy) is Static:
            proto_id_strategy.static.CopyFrom(
                cast(proto.Static, self.strategy.into_proto())
            )
        else:
            raise Exception("Encountered unknown type")
        return proto_id_strategy


@dataclasses.dataclass(frozen=True, slots=True,)
class IncrementOnlyUintProp(SerDe[proto.IncrementOnlyUintProp]):
    prop: int
    _proto_cls = proto.IncrementOnlyUintProp

    @classmethod
    def from_proto(
        cls,
        proto_increment_only_uint_prop: proto.IncrementOnlyUintProp,
    ) -> IncrementOnlyUintProp:
        return IncrementOnlyUintProp(prop=proto_increment_only_uint_prop.prop)

    def into_proto(self) -> proto.IncrementOnlyUintProp:
        proto_increment_only_uint_prop = proto.IncrementOnlyUintProp()
        proto_increment_only_uint_prop.prop = self.prop
        return proto_increment_only_uint_prop


@dataclasses.dataclass(frozen=True, slots=True,)
class ImmutableUintProp(SerDe[proto.ImmutableUintProp]):
    prop: int
    _proto_cls = proto.ImmutableUintProp

    @classmethod
    def from_proto(
        cls,
        proto_immutable_uint_prop: proto.ImmutableUintProp,
    ) -> ImmutableUintProp:
        return ImmutableUintProp(prop=proto_immutable_uint_prop.prop)

    def into_proto(self) -> proto.ImmutableUintProp:
        proto_immutable_uint_prop = proto.ImmutableUintProp()
        proto_immutable_uint_prop.prop = self.prop
        return proto_immutable_uint_prop


@dataclasses.dataclass(frozen=True, slots=True,)
class DecrementOnlyUintProp(SerDe[proto.DecrementOnlyUintProp]):
    prop: int
    _proto_cls = proto.DecrementOnlyUintProp

    @classmethod
    def from_proto(
        cls,
        proto_decrement_only_uint_prop: proto.DecrementOnlyUintProp,
    ) -> DecrementOnlyUintProp:
        return DecrementOnlyUintProp(prop=proto_decrement_only_uint_prop.prop)

    def into_proto(self) -> proto.DecrementOnlyUintProp:
        proto_decrement_only_uint_prop = proto.DecrementOnlyUintProp()
        proto_decrement_only_uint_prop.prop = self.prop
        return proto_decrement_only_uint_prop


@dataclasses.dataclass(frozen=True, slots=True,)
class IncrementOnlyIntProp(SerDe[proto.IncrementOnlyIntProp]):
    prop: int
    _proto_cls = proto.IncrementOnlyIntProp

    @classmethod
    def from_proto(
        cls,
        proto_increment_only_int_prop: proto.IncrementOnlyIntProp,
    ) -> IncrementOnlyIntProp:
        return IncrementOnlyIntProp(prop=proto_increment_only_int_prop.prop)

    def into_proto(self) -> proto.IncrementOnlyIntProp:
        proto_increment_only_int_prop = proto.IncrementOnlyIntProp()
        proto_increment_only_int_prop.prop = self.prop
        return proto_increment_only_int_prop


@dataclasses.dataclass(frozen=True, slots=True,)
class DecrementOnlyIntProp(SerDe[proto.DecrementOnlyIntProp]):
    prop: int
    _proto_cls = proto.DecrementOnlyIntProp

    @classmethod
    def from_proto(
        cls,
        proto_decrement_only_int_prop: proto.DecrementOnlyIntProp,
    ) -> DecrementOnlyIntProp:
        return DecrementOnlyIntProp(prop=proto_decrement_only_int_prop.prop)

    def into_proto(self) -> proto.DecrementOnlyIntProp:
        proto_decrement_only_int_prop = proto.DecrementOnlyIntProp()
        proto_decrement_only_int_prop.prop = self.prop
        return proto_decrement_only_int_prop


@dataclasses.dataclass(frozen=True, slots=True,)
class ImmutableIntProp(SerDe[proto.ImmutableIntProp]):
    prop: int
    _proto_cls = proto.ImmutableIntProp

    @classmethod
    def from_proto(
        cls,
        proto_immutable_int_prop: proto.ImmutableIntProp,
    ) -> ImmutableIntProp:
        return ImmutableIntProp(prop=proto_immutable_int_prop.prop)

    def into_proto(self) -> proto.ImmutableIntProp:
        proto_immutable_int_prop = proto.ImmutableIntProp()
        proto_immutable_int_prop.prop = self.prop
        return proto_immutable_int_prop


@dataclasses.dataclass(frozen=True, slots=True,)
class ImmutableStrProp(SerDe[proto.ImmutableStrProp]):
    prop: str
    _proto_cls = proto.ImmutableStrProp

    @classmethod
    def from_proto(
        cls,
        proto_immutable_str_prop: proto.ImmutableStrProp,
    ) -> ImmutableStrProp:
        return ImmutableStrProp(prop=proto_immutable_str_prop.prop)

    def into_proto(self) -> proto.ImmutableStrProp:
        proto_immutable_str_prop = proto.ImmutableStrProp()
        proto_immutable_str_prop.prop = self.prop
        return proto_immutable_str_prop


@dataclasses.dataclass(frozen=True, slots=True,)
class NodeProperty(SerDe[proto.NodeProperty]):
    property_: (
        IncrementOnlyUintProp
        | DecrementOnlyUintProp
        | ImmutableUintProp
        | IncrementOnlyIntProp
        | DecrementOnlyIntProp
        | ImmutableIntProp
        | ImmutableStrProp
    )
    _proto_cls = proto.NodeProperty

    @classmethod
    def from_proto(cls, proto_node_property: proto.NodeProperty) -> NodeProperty:
        if proto_node_property.HasField("increment_only_uint"):
            return NodeProperty(
                property_=IncrementOnlyUintProp.from_proto(
                    proto_node_property.increment_only_uint
                )
            )
        elif proto_node_property.HasField("decrement_only_uint"):
            return NodeProperty(
                property_=DecrementOnlyUintProp.from_proto(
                    proto_node_property.decrement_only_uint
                )
            )
        elif proto_node_property.HasField("immutable_uint"):
            return NodeProperty(
                property_=ImmutableUintProp.from_proto(
                    proto_node_property.immutable_uint
                )
            )
        elif proto_node_property.HasField("increment_only_int"):
            return NodeProperty(
                property_=IncrementOnlyIntProp.from_proto(
                    proto_node_property.increment_only_int
                )
            )
        elif proto_node_property.HasField("decrement_only_int"):
            return NodeProperty(
                property_=DecrementOnlyIntProp.from_proto(
                    proto_node_property.decrement_only_int
                )
            )
        elif proto_node_property.HasField("immutable_int"):
            return NodeProperty(
                property_=ImmutableIntProp.from_proto(proto_node_property.immutable_int)
            )
        elif proto_node_property.HasField("immutable_str"):
            return NodeProperty(
                property_=ImmutableStrProp.from_proto(proto_node_property.immutable_str)
            )
        else:
            raise Exception("Encountered unknown type")

    def into_proto(self) -> proto.NodeProperty:
        proto_node_property = proto.NodeProperty()
        if type(self.property_) is IncrementOnlyUintProp:
            proto_node_property.increment_only_uint.CopyFrom(
                cast(proto.IncrementOnlyUintProp, self.property_.into_proto())
            )
        elif type(self.property_) is DecrementOnlyUintProp:
            proto_node_property.decrement_only_uint.CopyFrom(
                cast(proto.DecrementOnlyUintProp, self.property_.into_proto())
            )
        elif type(self.property_) is ImmutableUintProp:
            proto_node_property.immutable_uint.CopyFrom(
                cast(proto.ImmutableUintProp, self.property_.into_proto())
            )
        elif type(self.property_) is IncrementOnlyIntProp:
            proto_node_property.increment_only_int.CopyFrom(
                cast(proto.IncrementOnlyIntProp, self.property_.into_proto())
            )
        elif type(self.property_) is DecrementOnlyIntProp:
            proto_node_property.decrement_only_int.CopyFrom(
                cast(proto.DecrementOnlyIntProp, self.property_.into_proto())
            )
        elif type(self.property_) is ImmutableIntProp:
            proto_node_property.immutable_int.CopyFrom(
                cast(proto.ImmutableIntProp, self.property_.into_proto())
            )
        elif type(self.property_) is ImmutableStrProp:
            proto_node_property.immutable_str.CopyFrom(
                cast(proto.ImmutableStrProp, self.property_.into_proto())
            )
        else:
            raise Exception("Encountered unknown type")
        return proto_node_property


@dataclasses.dataclass(frozen=True, slots=True,)
class NodeDescription(SerDe[proto.NodeDescription]):
    properties: Mapping[str, NodeProperty]
    node_key: str
    node_type: str
    id_strategy: Sequence[IdStrategy]
    _proto_cls = proto.NodeDescription

    @classmethod
    def from_proto(
        cls, proto_node_description: proto.NodeDescription
    ) -> NodeDescription:
        return NodeDescription(
            properties={
                k: NodeProperty.from_proto(proto_node_description.properties[k])
                for k in proto_node_description.properties
            },
            node_key=proto_node_description.node_key,
            node_type=proto_node_description.node_type,
            id_strategy=[
                IdStrategy.from_proto(s) for s in proto_node_description.id_strategy
            ],
        )

    def into_proto(self) -> proto.NodeDescription:
        proto_node_description = proto.NodeDescription()
        for k, v in self.properties.items():
            proto_node_description.properties[k].CopyFrom(v.into_proto())
        proto_node_description.node_key = self.node_key
        proto_node_description.node_type = self.node_type
        for s in self.id_strategy:
            proto_node_description.id_strategy.append(s.into_proto())
        return proto_node_description


@dataclasses.dataclass(frozen=True, slots=True,)
class IdentifiedNode(SerDe[proto.IdentifiedNode]):
    properties: Mapping[str, NodeProperty]
    uid: Uid
    node_type: str
    _proto_cls = proto.IdentifiedNode

    @classmethod
    def from_proto(cls, proto_identified_node: proto.IdentifiedNode) -> IdentifiedNode:
        return IdentifiedNode(
            properties={
                k: NodeProperty.from_proto(proto_identified_node.properties[k])
                for k in proto_identified_node.properties
            },
            uid=Uid.from_proto(proto_identified_node.uid),
            node_type=proto_identified_node.node_type,
        )

    def into_proto(self) -> proto.IdentifiedNode:
        proto_identified_node = proto.IdentifiedNode()
        for k, v in self.properties.items():
            proto_identified_node.properties[k].CopyFrom(v.into_proto())
        proto_identified_node.uid.CopyFrom(self.uid.into_proto())
        proto_identified_node.node_type = self.node_type
        return proto_identified_node


@dataclasses.dataclass(frozen=True, slots=True,)
class Edge(SerDe[proto.Edge]):
    from_node_key: str
    to_node_key: str
    edge_name: str
    _proto_cls = proto.Edge

    @classmethod
    def from_proto(cls, proto_edge: proto.Edge) -> Edge:
        return Edge(
            from_node_key=proto_edge.from_node_key,
            to_node_key=proto_edge.to_node_key,
            edge_name=proto_edge.edge_name,
        )

    def into_proto(self) -> proto.Edge:
        proto_edge = proto.Edge()
        proto_edge.from_node_key = self.from_node_key
        proto_edge.to_node_key = self.to_node_key
        proto_edge.edge_name = self.edge_name
        return proto_edge


@dataclasses.dataclass(frozen=True, slots=True,)
class EdgeList(SerDe[proto.EdgeList]):
    edges: Sequence[Edge]
    _proto_cls = proto.EdgeList

    @classmethod
    def from_proto(cls, proto_edge_list: proto.EdgeList) -> EdgeList:
        return EdgeList(edges=[Edge.from_proto(e) for e in proto_edge_list.edges])

    def into_proto(self) -> proto.EdgeList:
        proto_edge_list = proto.EdgeList()
        for e in self.edges:
            proto_edge_list.edges.append(e.into_proto())
        return proto_edge_list


@dataclasses.dataclass(frozen=True, slots=True,)
class IdentifiedEdge(SerDe[proto.IdentifiedEdge]):
    from_uid: Uid
    to_uid: Uid
    edge_name: str
    _proto_cls = proto.IdentifiedEdge

    @classmethod
    def from_proto(cls, proto_identified_edge: proto.IdentifiedEdge) -> IdentifiedEdge:
        return IdentifiedEdge(
            from_uid=Uid.from_proto(proto_identified_edge.from_uid),
            to_uid=Uid.from_proto(proto_identified_edge.to_uid),
            edge_name=proto_identified_edge.edge_name,
        )

    def into_proto(self) -> proto.IdentifiedEdge:
        proto_identified_edge = proto.IdentifiedEdge()
        proto_identified_edge.from_uid.CopyFrom(self.from_uid.into_proto())
        proto_identified_edge.to_uid.CopyFrom(self.to_uid.into_proto())
        proto_identified_edge.edge_name = self.edge_name
        return proto_identified_edge


@dataclasses.dataclass(frozen=True, slots=True)
class IdentifiedEdgeList(SerDe[proto.IdentifiedEdgeList]):
    # TODO: seed to places where this is used:
    # /src/python/grapl_analyzerlib/grapl_analyzerlib/view_from_proto.py
    # /src/python/grapl_analyzerlib/grapl_analyzerlib/subgraph_view.py
    edges: Sequence[IdentifiedEdge]
    _proto_cls = proto.IdentifiedEdgeList

    @classmethod
    def from_proto(
        cls, proto_identified_edge_list: proto.IdentifiedEdgeList
    ) -> IdentifiedEdgeList:
        return IdentifiedEdgeList(
            edges=[
                IdentifiedEdge.from_proto(e) for e in proto_identified_edge_list.edges
            ]
        )

    def into_proto(self) -> proto.IdentifiedEdgeList:
        proto_identified_edge_list = proto.IdentifiedEdgeList()
        for e in self.edges:
            proto_identified_edge_list.edges.append(e.into_proto())
        return proto_identified_edge_list


@dataclasses.dataclass(frozen=True, slots=True,)
class GraphDescription(SerDe[proto.GraphDescription]):
    nodes: Mapping[str, NodeDescription]
    edges: Mapping[str, EdgeList]
    _proto_cls = proto.GraphDescription

    @classmethod
    def from_proto(
        cls, proto_graph_description: proto.GraphDescription
    ) -> GraphDescription:
        return GraphDescription(
            nodes={
                k: NodeDescription.from_proto(proto_graph_description.nodes[k])
                for k in proto_graph_description.nodes
            },
            edges={
                k: EdgeList.from_proto(proto_graph_description.edges[k])
                for k in proto_graph_description.edges
            },
        )

    def into_proto(self) -> proto.GraphDescription:
        proto_graph_description = proto.GraphDescription()
        for k1, v1 in self.nodes.items():
            proto_graph_description.nodes[k1].CopyFrom(v1.into_proto())
        for k2, v2 in self.edges.items():
            proto_graph_description.edges[k2].CopyFrom(v2.into_proto())
        return proto_graph_description


@dataclasses.dataclass(frozen=True, slots=True,)
class IdentifiedGraph(SerDe[proto.IdentifiedGraph]):
    nodes: Mapping[Uid, IdentifiedNode]
    edges: Mapping[Uid, IdentifiedEdgeList]
    _proto_cls = proto.IdentifiedGraph

    @classmethod
    def from_proto(
        cls, proto_identified_graph: proto.IdentifiedGraph
    ) -> IdentifiedGraph:
        return IdentifiedGraph(
            nodes={
                Uid(k): IdentifiedNode.from_proto(proto_identified_graph.nodes[k])
                for k in proto_identified_graph.nodes
            },
            edges={
                Uid(k): IdentifiedEdgeList.from_proto(proto_identified_graph.edges[k])
                for k in proto_identified_graph.edges
            },
        )

    def into_proto(self) -> proto.IdentifiedGraph:
        proto_identified_graph = proto.IdentifiedGraph()
        for k1, v1 in self.nodes.items():
            proto_identified_graph.nodes[k1.value].CopyFrom(v1.into_proto())
        for k2, v2 in self.edges.items():
            proto_identified_graph.edges[k2.value].CopyFrom(v2.into_proto())
        return proto_identified_graph


@dataclasses.dataclass(frozen=True, slots=True,)
class Lens(SerDe[proto.Lens]):
    lens_type: str
    lens_name: str
    uid: int | None = None
    score: int | None = None
    _proto_cls = proto.Lens

    @classmethod
    def from_proto(cls, proto_lens: proto.Lens) -> Lens:
        return Lens(
            lens_type=proto_lens.lens_type,
            lens_name=proto_lens.lens_name,
            uid=proto_lens.uid,
            score=proto_lens.score,
        )

    def into_proto(self) -> proto.Lens:
        proto_lens = proto.Lens()
        proto_lens.lens_type = self.lens_type
        proto_lens.lens_name = self.lens_name
        if self.uid is not None:
            proto_lens.uid = self.uid
        if self.score is not None:
            proto_lens.score = self.score
        return proto_lens


@dataclasses.dataclass(frozen=True, slots=True,)
class ExecutionHit(SerDe[proto.ExecutionHit]):
    nodes: Mapping[Uid, IdentifiedNode]
    edges: Mapping[Uid, IdentifiedEdgeList]
    analyzer_name: str
    risk_score: int
    lenses: Sequence[Lens]
    risky_node_keys: Sequence[str]
    _proto_cls = proto.ExecutionHit

    @classmethod
    def from_proto(cls, proto_execution_hit: proto.ExecutionHit) -> ExecutionHit:
        return ExecutionHit(
            nodes={
                Uid(k): IdentifiedNode.from_proto(v)
                for k, v in proto_execution_hit.nodes.items()
            },
            edges={
                Uid(k): IdentifiedEdgeList.from_proto(v)
                for k, v in proto_execution_hit.edges.items()
            },
            analyzer_name=proto_execution_hit.analyzer_name,
            risk_score=proto_execution_hit.risk_score,
            lenses=[Lens.from_proto(l) for l in proto_execution_hit.lenses],
            risky_node_keys=proto_execution_hit.risky_node_keys,
        )

    def into_proto(self) -> proto.ExecutionHit:
        proto_execution_hit = proto.ExecutionHit()
        for k1, v1 in self.nodes.items():
            proto_execution_hit.nodes[k1.value].CopyFrom(v1.into_proto())
        for k2, v2 in self.edges.items():
            proto_execution_hit.edges[k2.value].CopyFrom(v2.into_proto())
        proto_execution_hit.analyzer_name = self.analyzer_name
        proto_execution_hit.risk_score = self.risk_score
        for lens in self.lenses:
            proto_execution_hit.lenses.append(lens.into_proto())
        for risky_node_key in self.risky_node_keys:
            proto_execution_hit.risky_node_keys.append(risky_node_key)
        return proto_execution_hit
