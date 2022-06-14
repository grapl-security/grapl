from __future__ import annotations

import dataclasses
from typing import Mapping, Sequence, Type, Union, cast

from graplinc.grapl.api.graph.v1beta1.types_pb2 import (
    DecrementOnlyIntProp as _DecrementOnlyIntProp,
)
from graplinc.grapl.api.graph.v1beta1.types_pb2 import (
    DecrementOnlyUintProp as _DecrementOnlyUintProp,
)
from graplinc.grapl.api.graph.v1beta1.types_pb2 import Edge as _Edge
from graplinc.grapl.api.graph.v1beta1.types_pb2 import EdgeList as _EdgeList
from graplinc.grapl.api.graph.v1beta1.types_pb2 import ExecutionHit as _ExecutionHit
from graplinc.grapl.api.graph.v1beta1.types_pb2 import (
    GraphDescription as _GraphDescription,
)
from graplinc.grapl.api.graph.v1beta1.types_pb2 import (
    IdentifiedGraph as _IdentifiedGraph,
)
from graplinc.grapl.api.graph.v1beta1.types_pb2 import IdentifiedNode as _IdentifiedNode
from graplinc.grapl.api.graph.v1beta1.types_pb2 import IdStrategy as _IdStrategy
from graplinc.grapl.api.graph.v1beta1.types_pb2 import (
    ImmutableIntProp as _ImmutableIntProp,
)
from graplinc.grapl.api.graph.v1beta1.types_pb2 import (
    ImmutableStrProp as _ImmutableStrProp,
)
from graplinc.grapl.api.graph.v1beta1.types_pb2 import (
    ImmutableUintProp as _ImmutableUintProp,
)
from graplinc.grapl.api.graph.v1beta1.types_pb2 import (
    IncrementOnlyIntProp as _IncrementOnlyIntProp,
)
from graplinc.grapl.api.graph.v1beta1.types_pb2 import (
    IncrementOnlyUintProp as _IncrementOnlyUintProp,
)
from graplinc.grapl.api.graph.v1beta1.types_pb2 import Lens as _Lens
from graplinc.grapl.api.graph.v1beta1.types_pb2 import MergedEdge as _MergedEdge
from graplinc.grapl.api.graph.v1beta1.types_pb2 import MergedEdgeList as _MergedEdgeList
from graplinc.grapl.api.graph.v1beta1.types_pb2 import MergedGraph as _MergedGraph
from graplinc.grapl.api.graph.v1beta1.types_pb2 import MergedNode as _MergedNode
from graplinc.grapl.api.graph.v1beta1.types_pb2 import (
    NodeDescription as _NodeDescription,
)
from graplinc.grapl.api.graph.v1beta1.types_pb2 import NodeProperty as _NodeProperty
from graplinc.grapl.api.graph.v1beta1.types_pb2 import Session as _Session
from graplinc.grapl.api.graph.v1beta1.types_pb2 import Static as _Static
from graplinc.grapl.api.suspicious_svchost_analyzer.v1beta1.suspicious_svchost_analyzer_pb2 import (
    AnalyzeRequest as _AnalyzeRequest,
)
from graplinc.grapl.api.suspicious_svchost_analyzer.v1beta1.suspicious_svchost_analyzer_pb2 import (
    AnalyzeResponse as _AnalyzeResponse,
)
from python_proto import SerDe


@dataclasses.dataclass(frozen=True)
class Session(SerDe[_Session]):
    primary_key_properties: Sequence[str]
    primary_key_requires_asset_id: bool
    create_time: int
    last_seen_time: int
    terminate_time: int
    proto_cls: Type[_Session] = _Session

    @staticmethod
    def deserialize(bytes_: bytes) -> Session:
        proto_session = _Session()
        proto_session.ParseFromString(bytes_)
        return Session.from_proto(proto_session=proto_session)

    @staticmethod
    def from_proto(proto_session: _Session) -> Session:
        return Session(
            primary_key_properties=proto_session.primary_key_properties,
            primary_key_requires_asset_id=proto_session.primary_key_requires_asset_id,
            create_time=proto_session.create_time,
            last_seen_time=proto_session.last_seen_time,
            terminate_time=proto_session.terminate_time,
        )

    def into_proto(self) -> _Session:
        proto_session = _Session()
        for prop in self.primary_key_properties:
            proto_session.primary_key_properties.append(prop)
        proto_session.primary_key_requires_asset_id = self.primary_key_requires_asset_id
        proto_session.create_time = self.create_time
        proto_session.last_seen_time = self.last_seen_time
        proto_session.terminate_time = self.terminate_time
        return proto_session


@dataclasses.dataclass(frozen=True)
class Static(SerDe[_Static]):
    primary_key_properties: Sequence[str]
    primary_key_requires_asset_id: bool
    proto_cls: Type[_Static] = _Static

    @staticmethod
    def deserialize(bytes_: bytes) -> Static:
        proto_static = _Static()
        proto_static.ParseFromString(bytes_)
        return Static.from_proto(proto_static=proto_static)

    @staticmethod
    def from_proto(proto_static: _Static) -> Static:
        return Static(
            primary_key_properties=proto_static.primary_key_properties,
            primary_key_requires_asset_id=proto_static.primary_key_requires_asset_id,
        )

    def into_proto(self) -> _Static:
        proto_static = _Static()
        for prop in self.primary_key_properties:
            proto_static.primary_key_properties.append(prop)
        proto_static.primary_key_requires_asset_id = self.primary_key_requires_asset_id
        return proto_static


@dataclasses.dataclass(frozen=True)
class IdStrategy(SerDe[_IdStrategy]):
    strategy: Union[Session, Static]
    proto_cls: Type[_IdStrategy] = _IdStrategy

    @staticmethod
    def deserialize(bytes_: bytes) -> IdStrategy:
        proto_id_strategy = _IdStrategy()
        proto_id_strategy.ParseFromString(bytes_)
        return IdStrategy.from_proto(proto_id_strategy=proto_id_strategy)

    @staticmethod
    def from_proto(proto_id_strategy: _IdStrategy) -> IdStrategy:
        if proto_id_strategy.HasField("session"):
            return IdStrategy(strategy=Session.from_proto(proto_id_strategy.session))
        elif proto_id_strategy.HasField("static"):
            return IdStrategy(strategy=Static.from_proto(proto_id_strategy.static))
        else:
            raise Exception("Encountered unknown type")

    def into_proto(self) -> _IdStrategy:
        proto_id_strategy = _IdStrategy()
        if type(self.strategy) is Session:
            proto_id_strategy.session.CopyFrom(
                cast(_Session, self.strategy.into_proto())
            )
        elif type(self.strategy) is Static:
            proto_id_strategy.static.CopyFrom(cast(_Static, self.strategy.into_proto()))
        else:
            raise Exception("Encountered unknown type")
        return proto_id_strategy


@dataclasses.dataclass(frozen=True)
class IncrementOnlyUintProp(SerDe[_IncrementOnlyUintProp]):
    prop: int
    proto_cls: Type[_IncrementOnlyUintProp] = _IncrementOnlyUintProp

    @staticmethod
    def deserialize(bytes_: bytes) -> IncrementOnlyUintProp:
        proto_increment_only_uint_prop = _IncrementOnlyUintProp()
        proto_increment_only_uint_prop.ParseFromString(bytes_)
        return IncrementOnlyUintProp.from_proto(
            proto_increment_only_uint_prop=proto_increment_only_uint_prop
        )

    @staticmethod
    def from_proto(
        proto_increment_only_uint_prop: _IncrementOnlyUintProp,
    ) -> IncrementOnlyUintProp:
        return IncrementOnlyUintProp(prop=proto_increment_only_uint_prop.prop)

    def into_proto(self) -> _IncrementOnlyUintProp:
        proto_increment_only_uint_prop = _IncrementOnlyUintProp()
        proto_increment_only_uint_prop.prop = self.prop
        return proto_increment_only_uint_prop


@dataclasses.dataclass(frozen=True)
class ImmutableUintProp(SerDe[_ImmutableUintProp]):
    prop: int
    proto_cls: Type[_ImmutableUintProp] = _ImmutableUintProp

    @staticmethod
    def deserialize(bytes_: bytes) -> ImmutableUintProp:
        proto_immutable_uint_prop = _ImmutableUintProp()
        proto_immutable_uint_prop.ParseFromString(bytes_)
        return ImmutableUintProp.from_proto(
            proto_immutable_uint_prop=proto_immutable_uint_prop
        )

    @staticmethod
    def from_proto(proto_immutable_uint_prop: _ImmutableUintProp) -> ImmutableUintProp:
        return ImmutableUintProp(prop=proto_immutable_uint_prop.prop)

    def into_proto(self) -> _ImmutableUintProp:
        proto_immutable_uint_prop = _ImmutableUintProp()
        proto_immutable_uint_prop.prop = self.prop
        return proto_immutable_uint_prop


@dataclasses.dataclass(frozen=True)
class DecrementOnlyUintProp(SerDe[_DecrementOnlyUintProp]):
    prop: int
    proto_cls: Type[_DecrementOnlyUintProp] = _DecrementOnlyUintProp

    @staticmethod
    def deserialize(bytes_: bytes) -> DecrementOnlyUintProp:
        proto_decrement_only_uint_prop = _DecrementOnlyUintProp()
        proto_decrement_only_uint_prop.ParseFromString(bytes_)
        return DecrementOnlyUintProp.from_proto(
            proto_decrement_only_uint_prop=proto_decrement_only_uint_prop
        )

    @staticmethod
    def from_proto(
        proto_decrement_only_uint_prop: _DecrementOnlyUintProp,
    ) -> DecrementOnlyUintProp:
        return DecrementOnlyUintProp(prop=proto_decrement_only_uint_prop.prop)

    def into_proto(self) -> _DecrementOnlyUintProp:
        proto_decrement_only_uint_prop = _DecrementOnlyUintProp()
        proto_decrement_only_uint_prop.prop = self.prop
        return proto_decrement_only_uint_prop


@dataclasses.dataclass(frozen=True)
class IncrementOnlyIntProp(SerDe[_IncrementOnlyIntProp]):
    prop: int
    proto_cls: Type[_IncrementOnlyIntProp] = _IncrementOnlyIntProp

    @staticmethod
    def deserialize(bytes_: bytes) -> IncrementOnlyIntProp:
        proto_increment_only_int_prop = _IncrementOnlyIntProp()
        proto_increment_only_int_prop.ParseFromString(bytes_)
        return IncrementOnlyIntProp.from_proto(
            proto_increment_only_int_prop=proto_increment_only_int_prop
        )

    @staticmethod
    def from_proto(
        proto_increment_only_int_prop: _IncrementOnlyIntProp,
    ) -> IncrementOnlyIntProp:
        return IncrementOnlyIntProp(prop=proto_increment_only_int_prop.prop)

    def into_proto(self) -> _IncrementOnlyIntProp:
        proto_increment_only_int_prop = _IncrementOnlyIntProp()
        proto_increment_only_int_prop.prop = self.prop
        return proto_increment_only_int_prop


@dataclasses.dataclass(frozen=True)
class DecrementOnlyIntProp(SerDe[_DecrementOnlyIntProp]):
    prop: int
    proto_cls: Type[_DecrementOnlyIntProp] = _DecrementOnlyIntProp

    @staticmethod
    def deserialize(bytes_: bytes) -> DecrementOnlyIntProp:
        proto_decrement_only_int_prop = _DecrementOnlyIntProp()
        proto_decrement_only_int_prop.ParseFromString(bytes_)
        return DecrementOnlyIntProp.from_proto(
            proto_decrement_only_int_prop=proto_decrement_only_int_prop
        )

    @staticmethod
    def from_proto(
        proto_decrement_only_int_prop: _DecrementOnlyIntProp,
    ) -> DecrementOnlyIntProp:
        return DecrementOnlyIntProp(prop=proto_decrement_only_int_prop.prop)

    def into_proto(self) -> _DecrementOnlyIntProp:
        proto_decrement_only_int_prop = _DecrementOnlyIntProp()
        proto_decrement_only_int_prop.prop = self.prop
        return proto_decrement_only_int_prop


@dataclasses.dataclass(frozen=True)
class ImmutableIntProp(SerDe[_ImmutableIntProp]):
    prop: int
    proto_cls: Type[_ImmutableIntProp] = _ImmutableIntProp

    @staticmethod
    def deserialize(bytes_: bytes) -> ImmutableIntProp:
        proto_immutable_int_prop = _ImmutableIntProp()
        proto_immutable_int_prop.ParseFromString(bytes_)
        return ImmutableIntProp.from_proto(
            proto_immutable_int_prop=proto_immutable_int_prop
        )

    @staticmethod
    def from_proto(proto_immutable_int_prop: _ImmutableIntProp) -> ImmutableIntProp:
        return ImmutableIntProp(prop=proto_immutable_int_prop.prop)

    def into_proto(self) -> _ImmutableIntProp:
        proto_immutable_int_prop = _ImmutableIntProp()
        proto_immutable_int_prop.prop = self.prop
        return proto_immutable_int_prop


@dataclasses.dataclass(frozen=True)
class ImmutableStrProp(SerDe[_ImmutableStrProp]):
    prop: str
    proto_cls: Type[_ImmutableStrProp] = _ImmutableStrProp

    @staticmethod
    def deserialize(bytes_: bytes) -> ImmutableStrProp:
        proto_immutable_str_prop = _ImmutableStrProp()
        proto_immutable_str_prop.ParseFromString(bytes_)
        return ImmutableStrProp.from_proto(
            proto_immutable_str_prop=proto_immutable_str_prop
        )

    @staticmethod
    def from_proto(proto_immutable_str_prop: _ImmutableStrProp) -> ImmutableStrProp:
        return ImmutableStrProp(prop=proto_immutable_str_prop.prop)

    def into_proto(self) -> _ImmutableStrProp:
        proto_immutable_str_prop = _ImmutableStrProp()
        proto_immutable_str_prop.prop = self.prop
        return proto_immutable_str_prop


@dataclasses.dataclass(frozen=True)
class NodeProperty(SerDe[_NodeProperty]):
    property_: Union[
        IncrementOnlyUintProp,
        DecrementOnlyUintProp,
        ImmutableUintProp,
        IncrementOnlyIntProp,
        DecrementOnlyIntProp,
        ImmutableIntProp,
        ImmutableStrProp,
    ]
    proto_cls: Type[_NodeProperty] = _NodeProperty

    @staticmethod
    def deserialize(bytes_: bytes) -> NodeProperty:
        proto_node_property = _NodeProperty()
        proto_node_property.ParseFromString(bytes_)
        return NodeProperty.from_proto(proto_node_property=proto_node_property)

    @staticmethod
    def from_proto(proto_node_property: _NodeProperty) -> NodeProperty:
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

    def into_proto(self) -> _NodeProperty:
        proto_node_property = _NodeProperty()
        if type(self.property_) is IncrementOnlyUintProp:
            proto_node_property.increment_only_uint.CopyFrom(
                cast(_IncrementOnlyUintProp, self.property_.into_proto())
            )
        elif type(self.property_) is DecrementOnlyUintProp:
            proto_node_property.decrement_only_uint.CopyFrom(
                cast(_DecrementOnlyUintProp, self.property_.into_proto())
            )
        elif type(self.property_) is ImmutableUintProp:
            proto_node_property.immutable_uint.CopyFrom(
                cast(_ImmutableUintProp, self.property_.into_proto())
            )
        elif type(self.property_) is IncrementOnlyIntProp:
            proto_node_property.increment_only_int.CopyFrom(
                cast(_IncrementOnlyIntProp, self.property_.into_proto())
            )
        elif type(self.property_) is DecrementOnlyIntProp:
            proto_node_property.decrement_only_int.CopyFrom(
                cast(_DecrementOnlyIntProp, self.property_.into_proto())
            )
        elif type(self.property_) is ImmutableIntProp:
            proto_node_property.immutable_int.CopyFrom(
                cast(_ImmutableIntProp, self.property_.into_proto())
            )
        elif type(self.property_) is ImmutableStrProp:
            proto_node_property.immutable_str.CopyFrom(
                cast(_ImmutableStrProp, self.property_.into_proto())
            )
        else:
            raise Exception("Encountered unknown type")
        return proto_node_property


@dataclasses.dataclass(frozen=True)
class NodeDescription(SerDe[_NodeDescription]):
    properties: Mapping[str, NodeProperty]
    node_key: str
    node_type: str
    id_strategy: Sequence[IdStrategy]
    proto_cls: Type[_NodeDescription] = _NodeDescription

    @staticmethod
    def deserialize(bytes_: bytes) -> NodeDescription:
        proto_node_description = _NodeDescription()
        proto_node_description.ParseFromString(bytes_)
        return NodeDescription.from_proto(proto_node_description=proto_node_description)

    @staticmethod
    def from_proto(proto_node_description: _NodeDescription) -> NodeDescription:
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

    def into_proto(self) -> _NodeDescription:
        proto_node_description = _NodeDescription()
        for k, v in self.properties.items():
            proto_node_description.properties[k].CopyFrom(v.into_proto())
        proto_node_description.node_key = self.node_key
        proto_node_description.node_type = self.node_type
        for s in self.id_strategy:
            proto_node_description.id_strategy.append(s.into_proto())
        return proto_node_description


@dataclasses.dataclass(frozen=True)
class IdentifiedNode(SerDe[_IdentifiedNode]):
    properties: Mapping[str, NodeProperty]
    node_key: str
    node_type: str
    proto_cls: Type[_IdentifiedNode] = _IdentifiedNode

    @staticmethod
    def deserialize(bytes_: bytes) -> IdentifiedNode:
        proto_identified_node = _IdentifiedNode()
        proto_identified_node.ParseFromString(bytes_)
        return IdentifiedNode.from_proto(proto_identified_node=proto_identified_node)

    @staticmethod
    def from_proto(proto_identified_node: _IdentifiedNode) -> IdentifiedNode:
        return IdentifiedNode(
            properties={
                k: NodeProperty.from_proto(proto_identified_node.properties[k])
                for k in proto_identified_node.properties
            },
            node_key=proto_identified_node.node_key,
            node_type=proto_identified_node.node_type,
        )

    def into_proto(self) -> _IdentifiedNode:
        proto_identified_node = _IdentifiedNode()
        for k, v in self.properties.items():
            proto_identified_node.properties[k].CopyFrom(v.into_proto())
        proto_identified_node.node_key = self.node_key
        proto_identified_node.node_type = self.node_type
        return proto_identified_node


@dataclasses.dataclass(frozen=True)
class MergedNode(SerDe[_MergedNode]):
    properties: Mapping[str, NodeProperty]
    uid: int
    node_key: str
    node_type: str
    proto_cls: Type[_MergedNode] = _MergedNode

    @staticmethod
    def deserialize(bytes_: bytes) -> MergedNode:
        proto_merged_node = _MergedNode()
        proto_merged_node.ParseFromString(bytes_)
        return MergedNode.from_proto(proto_merged_node=proto_merged_node)

    @staticmethod
    def from_proto(proto_merged_node: _MergedNode) -> MergedNode:
        return MergedNode(
            properties={
                k: NodeProperty.from_proto(proto_merged_node.properties[k])
                for k in proto_merged_node.properties
            },
            uid=proto_merged_node.uid,
            node_key=proto_merged_node.node_key,
            node_type=proto_merged_node.node_type,
        )

    def into_proto(self) -> _MergedNode:
        proto_merged_node = _MergedNode()
        for k, v in self.properties.items():
            proto_merged_node.properties[k].CopyFrom(v.into_proto())
        proto_merged_node.uid = self.uid
        proto_merged_node.node_key = self.node_key
        proto_merged_node.node_type = self.node_type
        return proto_merged_node


@dataclasses.dataclass(frozen=True)
class Edge(SerDe[_Edge]):
    from_node_key: str
    to_node_key: str
    edge_name: str
    proto_cls: Type[_Edge] = _Edge

    @staticmethod
    def deserialize(bytes_: bytes) -> Edge:
        proto_edge = _Edge()
        proto_edge.ParseFromString(bytes_)
        return Edge.from_proto(proto_edge=proto_edge)

    @staticmethod
    def from_proto(proto_edge: _Edge) -> Edge:
        return Edge(
            from_node_key=proto_edge.from_node_key,
            to_node_key=proto_edge.to_node_key,
            edge_name=proto_edge.edge_name,
        )

    def into_proto(self) -> _Edge:
        proto_edge = _Edge()
        proto_edge.from_node_key = self.from_node_key
        proto_edge.to_node_key = self.to_node_key
        proto_edge.edge_name = self.edge_name
        return proto_edge


@dataclasses.dataclass(frozen=True)
class EdgeList(SerDe[_EdgeList]):
    edges: Sequence[Edge]
    proto_cls: Type[_EdgeList] = _EdgeList

    @staticmethod
    def deserialize(bytes_: bytes) -> EdgeList:
        proto_edge_list = _EdgeList()
        proto_edge_list.ParseFromString(bytes_)
        return EdgeList.from_proto(proto_edge_list=proto_edge_list)

    @staticmethod
    def from_proto(proto_edge_list: _EdgeList) -> EdgeList:
        return EdgeList(edges=[Edge.from_proto(e) for e in proto_edge_list.edges])

    def into_proto(self) -> _EdgeList:
        proto_edge_list = _EdgeList()
        for e in self.edges:
            proto_edge_list.edges.append(e.into_proto())
        return proto_edge_list


@dataclasses.dataclass(frozen=True)
class MergedEdge(SerDe[_MergedEdge]):
    from_uid: str
    from_node_key: str
    to_uid: str
    to_node_key: str
    edge_name: str
    proto_cls: Type[_MergedEdge] = _MergedEdge

    @staticmethod
    def deserialize(bytes_: bytes) -> MergedEdge:
        proto_merged_edge = _MergedEdge()
        proto_merged_edge.ParseFromString(bytes_)
        return MergedEdge.from_proto(proto_merged_edge=proto_merged_edge)

    @staticmethod
    def from_proto(proto_merged_edge: _MergedEdge) -> MergedEdge:
        return MergedEdge(
            from_uid=proto_merged_edge.from_uid,
            from_node_key=proto_merged_edge.from_node_key,
            to_uid=proto_merged_edge.to_uid,
            to_node_key=proto_merged_edge.to_node_key,
            edge_name=proto_merged_edge.edge_name,
        )

    def into_proto(self) -> _MergedEdge:
        proto_merged_edge = _MergedEdge()
        proto_merged_edge.from_uid = self.from_uid
        proto_merged_edge.from_node_key = self.from_node_key
        proto_merged_edge.to_uid = self.to_uid
        proto_merged_edge.to_node_key = self.to_node_key
        proto_merged_edge.edge_name = self.edge_name
        return proto_merged_edge


@dataclasses.dataclass(frozen=True)
class MergedEdgeList(SerDe[_MergedEdgeList]):
    # TODO: seed to places where this is used:
    # /src/python/grapl_analyzerlib/grapl_analyzerlib/view_from_proto.py
    # /src/python/grapl_analyzerlib/grapl_analyzerlib/subgraph_view.py
    edges: Sequence[MergedEdge]
    proto_cls: Type[_MergedEdgeList] = _MergedEdgeList

    @staticmethod
    def deserialize(bytes_: bytes) -> MergedEdgeList:
        proto_merged_edge_list = _MergedEdgeList()
        proto_merged_edge_list.ParseFromString(bytes_)
        return MergedEdgeList.from_proto(proto_merged_edge_list=proto_merged_edge_list)

    @staticmethod
    def from_proto(proto_merged_edge_list: _MergedEdgeList) -> MergedEdgeList:
        return MergedEdgeList(
            edges=[MergedEdge.from_proto(e) for e in proto_merged_edge_list.edges]
        )

    def into_proto(self) -> _MergedEdgeList:
        proto_merged_edge_list = _MergedEdgeList()
        for e in self.edges:
            proto_merged_edge_list.edges.append(e.into_proto())
        return proto_merged_edge_list


@dataclasses.dataclass(frozen=True)
class GraphDescription(SerDe[_GraphDescription]):
    nodes: Mapping[str, NodeDescription]
    edges: Mapping[str, EdgeList]
    proto_cls: Type[_GraphDescription] = _GraphDescription

    @staticmethod
    def deserialize(bytes_: bytes) -> GraphDescription:
        proto_graph_description = _GraphDescription()
        proto_graph_description.ParseFromString(bytes_)
        return GraphDescription.from_proto(
            proto_graph_description=proto_graph_description
        )

    @staticmethod
    def from_proto(proto_graph_description: _GraphDescription) -> GraphDescription:
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

    def into_proto(self) -> _GraphDescription:
        proto_graph_description = _GraphDescription()
        for k1, v1 in self.nodes.items():
            proto_graph_description.nodes[k1].CopyFrom(v1.into_proto())
        for k2, v2 in self.edges.items():
            proto_graph_description.edges[k2].CopyFrom(v2.into_proto())
        return proto_graph_description


@dataclasses.dataclass(frozen=True)
class IdentifiedGraph(SerDe[_IdentifiedGraph]):
    nodes: Mapping[str, IdentifiedNode]
    edges: Mapping[str, EdgeList]
    proto_cls: Type[_IdentifiedGraph] = _IdentifiedGraph

    @staticmethod
    def deserialize(bytes_: bytes) -> IdentifiedGraph:
        proto_identified_graph = _IdentifiedGraph()
        proto_identified_graph.ParseFromString(bytes_)
        return IdentifiedGraph.from_proto(proto_identified_graph=proto_identified_graph)

    @staticmethod
    def from_proto(proto_identified_graph: _IdentifiedGraph) -> IdentifiedGraph:
        return IdentifiedGraph(
            nodes={
                k: IdentifiedNode.from_proto(proto_identified_graph.nodes[k])
                for k in proto_identified_graph.nodes
            },
            edges={
                k: EdgeList.from_proto(proto_identified_graph.edges[k])
                for k in proto_identified_graph.edges
            },
        )

    def into_proto(self) -> _IdentifiedGraph:
        proto_identified_graph = _IdentifiedGraph()
        for k1, v1 in self.nodes.items():
            proto_identified_graph.nodes[k1].CopyFrom(v1.into_proto())
        for k2, v2 in self.edges.items():
            proto_identified_graph.edges[k2].CopyFrom(v2.into_proto())
        return proto_identified_graph


@dataclasses.dataclass(frozen=True)
class MergedGraph(SerDe[_MergedGraph]):
    nodes: Mapping[str, MergedNode]
    edges: Mapping[str, MergedEdgeList]
    proto_cls: Type[_MergedGraph] = _MergedGraph

    @staticmethod
    def deserialize(bytes_: bytes) -> MergedGraph:
        proto_merged_graph = _MergedGraph()
        proto_merged_graph.ParseFromString(bytes_)
        return MergedGraph.from_proto(proto_merged_graph=proto_merged_graph)

    @staticmethod
    def from_proto(proto_merged_graph: _MergedGraph) -> MergedGraph:
        return MergedGraph(
            nodes={
                k: MergedNode.from_proto(proto_merged_graph.nodes[k])
                for k in proto_merged_graph.nodes
            },
            edges={
                k: MergedEdgeList.from_proto(proto_merged_graph.edges[k])
                for k in proto_merged_graph.edges
            },
        )

    def into_proto(self) -> _MergedGraph:
        proto_merged_graph = _MergedGraph()
        for k1, v1 in self.nodes.items():
            proto_merged_graph.nodes[k1].CopyFrom(v1.into_proto())
        for k2, v2 in self.edges.items():
            proto_merged_graph.edges[k2].CopyFrom(v2.into_proto())
        return proto_merged_graph


@dataclasses.dataclass(frozen=True)
class AnalyzeRequest(SerDe[_AnalyzeRequest]):
    merged_graph: MergedGraph

    @staticmethod
    def deserialize(bytes_: bytes) -> AnalyzeRequest:
        proto_analyze_request = _AnalyzeRequest()
        proto_analyze_request.ParseFromString(bytes_)
        return AnalyzeRequest.from_proto(proto_analyze_request=proto_analyze_request)

    @staticmethod
    def from_proto(proto_analyze_request: _AnalyzeRequest) -> AnalyzeRequest:
        return AnalyzeRequest(
            merged_graph=MergedGraph.from_proto(
                proto_merged_graph=proto_analyze_request.merged_graph
            )
        )

    def into_proto(self) -> AnalyzeRequest:
        proto_analyze_request = _AnalyzeRequest()
        proto_analyze_request.merged_graph.CopyFrom(self.merged_graph.into_proto())
        return proto_analyze_request


@dataclasses.dataclass(frozen=True)
class Lens(SerDe[_Lens]):
    lens_type: str
    lens_name: str
    uid: int
    score: int

    @staticmethod
    def deserialize(bytes_: bytes) -> Lens:
        proto_lens = _Lens()
        proto_lens.ParseFromString(bytes_)
        return Lens.from_proto(proto_lens=proto_lens)

    @staticmethod
    def from_proto(proto_lens: _Lens) -> Lens:
        return Lens(
            lens_type=proto_lens.lens_type,
            lens_name=proto_lens.lens_name,
            uid=proto_lens.uid,
            score=proto_lens.score,
        )

    def into_proto(self) -> _Lens:
        proto_lens = _Lens()
        proto_lens.lens_type = self.lens_type
        proto_lens.lens_name = self.lens_name
        proto_lens.uid = self.uid
        proto_lens.score = self.score
        return proto_lens


@dataclasses.dataclass(frozen=True)
class ExecutionHit(SerDe[_ExecutionHit]):
    nodes: Mapping[str, MergedNode]
    edges: Mapping[str, MergedEdgeList]
    analyzer_name: str
    risk_score: int
    lenses: Sequence[Lens]
    risky_node_keys: Sequence[str]

    @staticmethod
    def deserialize(bytes_: bytes) -> ExecutionHit:
        proto_execution_hit = _ExecutionHit()
        proto_execution_hit.ParseFromString(bytes_)
        return ExecutionHit.from_proto(proto_execution_hit=proto_execution_hit)

    @staticmethod
    def from_proto(proto_execution_hit: _ExecutionHit) -> ExecutionHit:
        return ExecutionHit(
            nodes={
                k: MergedNode.from_proto(v)
                for k, v in proto_execution_hit.nodes.items()
            },
            edges={
                k: MergedEdgeList.from_proto(v)
                for k, v in proto_execution_hit.edges.items()
            },
            analyzer_name=proto_execution_hit.analyzer_name,
            risk_score=proto_execution_hit.risk_score,
            lenses=[Lens.from_proto(l) for l in proto_execution_hit.lenses],
            risky_node_keys=proto_execution_hit.risky_node_keys,
        )

    def into_proto(self) -> _ExecutionHit:
        proto_execution_hit = _ExecutionHit()
        for k1, v1 in self.nodes.items():
            proto_execution_hit.nodes[k1].CopyFrom(v1.into_proto())
        for k2, v2 in self.edges.items():
            proto_execution_hit.edges[k2].CopyFrom(v2.into_proto())
        proto_execution_hit.analyzer_name = self.analyzer_name
        proto_execution_hit.risk_score = self.risk_score
        for lens in self.lenses:
            proto_execution_hit.lenses.append(lens.into_proto())
        for risky_node_key in self.risky_node_keys:
            proto_execution_hit.risky_node_keys.append(risky_node_key)
        return proto_execution_hit


@dataclasses.dataclass(frozen=True)
class AnalyzeResponse(SerDe[_AnalyzeResponse]):
    execution_hits: Sequence[ExecutionHit]

    @staticmethod
    def deserialize(bytes_: bytes) -> AnalyzeResponse:
        proto_analyze_response = _AnalyzeResponse()
        proto_analyze_response.ParseFromString(bytes_)
        return AnalyzeResponse.from_proto(proto_analyze_response=proto_analyze_response)

    @staticmethod
    def from_proto(proto_analyze_response: _AnalyzeResponse) -> AnalyzeResponse:
        return AnalyzeResponse(
            execution_hits=[
                ExecutionHit.from_proto(h)
                for h in proto_analyze_response.execution_hits
            ]
        )

    def into_proto(self) -> _AnalyzeResponse:
        proto_analyze_response = _AnalyzeResponse()
        for execution_hit in self.execution_hits:
            proto_analyze_response.execution_hits.append(execution_hit.into_proto())
        return proto_analyze_response
