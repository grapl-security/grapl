from __future__ import annotations

import dataclasses

from graplinc.grapl.common.v1beta1 import types_pb2 as proto
from python_proto.serde import SerDe


@dataclasses.dataclass(frozen=True)
class Uid(SerDe[proto.Uid]):
    value: int

    proto_cls = proto.Uid

    @classmethod
    def from_proto(cls, proto_value: proto.Uid) -> Uid:
        return cls(value=proto_value.value)

    def into_proto(self) -> proto.Uid:
        proto_value = self.proto_cls()
        proto_value.value = self.value
        return proto_value


@dataclasses.dataclass(frozen=True)
class PropertyName(SerDe[proto.PropertyName]):
    value: str

    proto_cls = proto.PropertyName

    @classmethod
    def from_proto(cls, proto_value: proto.PropertyName) -> PropertyName:
        return cls(value=proto_value.value)

    def into_proto(self) -> proto.PropertyName:
        proto_value = self.proto_cls()
        proto_value.value = self.value
        return proto_value


@dataclasses.dataclass(frozen=True)
class EdgeName(SerDe[proto.EdgeName]):
    value: str

    proto_cls = proto.EdgeName

    @classmethod
    def from_proto(cls, proto_value: proto.EdgeName) -> EdgeName:
        return cls(value=proto_value.value)

    def into_proto(self) -> proto.EdgeName:
        proto_value = self.proto_cls()
        proto_value.value = self.value
        return proto_value


@dataclasses.dataclass(frozen=True)
class NodeType(SerDe[proto.NodeType]):
    value: str

    proto_cls = proto.NodeType

    @classmethod
    def from_proto(cls, proto_value: proto.NodeType) -> NodeType:
        return cls(value=proto_value.value)

    def into_proto(self) -> proto.NodeType:
        proto_value = self.proto_cls()
        proto_value.value = self.value
        return proto_value
