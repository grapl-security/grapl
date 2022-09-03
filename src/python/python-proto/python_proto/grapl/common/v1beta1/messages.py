from __future__ import annotations

import dataclasses

from graplinc.grapl.common.v1beta1 import types_pb2 as proto
from python_proto.serde import SerDe


@dataclasses.dataclass(frozen=True)
class Uid(SerDe[proto.Uid]):
    value: int

    proto_cls: type[proto.Uid] = proto.Uid

    @classmethod
    def from_proto(cls, proto_value: proto.Uid) -> Uid:
        return cls(value=proto_value.value)

    def into_proto(self) -> proto.Uid:
        proto_value = self.proto_cls()
        proto_value.value = self.value
        return proto_value


"""
syntax = "proto3";

package graplinc.grapl.common.v1beta1;

// A wrapper type for property names
message PropertyName {
  // The property name must:
  // - Be non-empty
  // - Snake case, `^[a-z]+(_[a-z]+)*$`
  // - Less than 32 characters
  string value = 1;
}

// A wrapper type for edge names
message EdgeName {
  // The edge name must:
  // - Be non-empty
  // - Snake case, `^[a-z]+(_[a-z]+)*$`
  // - Less than 32 characters
  string value = 1;
}

// A wrapper type for node type names
message NodeType {
  // The node type must:
  // - Be non-empty
  // - PascalCase, `^([A-Z][a-z]+)+$`
  // - Less than 32 characters
  string value = 1;
}

// A wrapper type for a node's uid
message Uid {
  // Can never be 0
  uint64 value = 1;
}

"""
