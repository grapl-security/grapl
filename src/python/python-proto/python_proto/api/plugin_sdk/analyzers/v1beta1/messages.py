from __future__ import annotations

import dataclasses

from graplinc.grapl.api.plugin_sdk.analyzers.v1beta1 import analyzers_pb2 as proto
from python_proto import common as proto_common_msgs
from python_proto.grapl.common.v1beta1 import messages as grapl_common_msgs
from python_proto.serde import SerDe


@dataclasses.dataclass(frozen=True, slots=True)
class RunAnalyzerRequest(SerDe[proto.RunAnalyzerRequest]):
    update: Update

    _proto_cls = proto.RunAnalyzerRequest

    @classmethod
    def from_proto(
        cls,
        proto_value: proto.RunAnalyzerRequest,
    ) -> RunAnalyzerRequest:
        return cls(
            update=Update.from_proto(proto_value.update),
        )

    def into_proto(self) -> proto.RunAnalyzerRequest:
        proto_value = self.new_proto()
        proto_value.update.CopyFrom(self.update.into_proto())
        return proto_value


@dataclasses.dataclass(frozen=True, slots=True)
class StringPropertyUpdate(SerDe[proto.StringPropertyUpdate]):
    uid: grapl_common_msgs.Uid
    property_name: grapl_common_msgs.PropertyName

    _proto_cls = proto.StringPropertyUpdate

    @classmethod
    def from_proto(
        cls,
        proto_value: proto.StringPropertyUpdate,
    ) -> StringPropertyUpdate:
        return cls(
            uid=grapl_common_msgs.Uid.from_proto(proto_value.uid),
            property_name=grapl_common_msgs.PropertyName.from_proto(
                proto_value.property_name
            ),
        )

    def into_proto(self) -> proto.StringPropertyUpdate:
        proto_value = self.new_proto()
        proto_value.uid.CopyFrom(self.uid.into_proto())
        proto_value.property_name.CopyFrom(self.property_name.into_proto())
        return proto_value


@dataclasses.dataclass(frozen=True, slots=True)
class UInt64PropertyUpdate(SerDe[proto.UInt64PropertyUpdate]):
    uid: grapl_common_msgs.Uid
    property_name: grapl_common_msgs.PropertyName

    _proto_cls = proto.UInt64PropertyUpdate

    @classmethod
    def from_proto(
        cls,
        proto_value: proto.UInt64PropertyUpdate,
    ) -> UInt64PropertyUpdate:
        return cls(
            uid=grapl_common_msgs.Uid.from_proto(proto_value.uid),
            property_name=grapl_common_msgs.PropertyName.from_proto(
                proto_value.property_name
            ),
        )

    def into_proto(self) -> proto.UInt64PropertyUpdate:
        proto_value = self.new_proto()
        proto_value.uid.CopyFrom(self.uid.into_proto())
        proto_value.property_name.CopyFrom(self.property_name.into_proto())
        return proto_value


@dataclasses.dataclass(frozen=True, slots=True)
class Int64PropertyUpdate(SerDe[proto.Int64PropertyUpdate]):
    uid: grapl_common_msgs.Uid
    property_name: grapl_common_msgs.PropertyName

    _proto_cls = proto.Int64PropertyUpdate

    @classmethod
    def from_proto(
        cls,
        proto_value: proto.Int64PropertyUpdate,
    ) -> Int64PropertyUpdate:
        return cls(
            uid=grapl_common_msgs.Uid.from_proto(proto_value.uid),
            property_name=grapl_common_msgs.PropertyName.from_proto(
                proto_value.property_name
            ),
        )

    def into_proto(self) -> proto.Int64PropertyUpdate:
        proto_value = self.new_proto()
        proto_value.uid.CopyFrom(self.uid.into_proto())
        proto_value.property_name.CopyFrom(self.property_name.into_proto())
        return proto_value


@dataclasses.dataclass(frozen=True, slots=True)
class EdgeUpdate(SerDe[proto.EdgeUpdate]):
    src_uid: grapl_common_msgs.Uid
    dst_uid: grapl_common_msgs.Uid
    forward_edge_name: grapl_common_msgs.EdgeName
    reverse_edge_name: grapl_common_msgs.EdgeName

    _proto_cls = proto.EdgeUpdate

    @classmethod
    def from_proto(
        cls,
        proto_value: proto.EdgeUpdate,
    ) -> EdgeUpdate:
        return cls(
            src_uid=grapl_common_msgs.Uid.from_proto(proto_value.src_uid),
            dst_uid=grapl_common_msgs.Uid.from_proto(proto_value.dst_uid),
            forward_edge_name=grapl_common_msgs.EdgeName.from_proto(
                proto_value.forward_edge_name
            ),
            reverse_edge_name=grapl_common_msgs.EdgeName.from_proto(
                proto_value.reverse_edge_name
            ),
        )

    def into_proto(self) -> proto.EdgeUpdate:
        proto_value = self.new_proto()
        proto_value.src_uid.CopyFrom(self.src_uid.into_proto())
        proto_value.dst_uid.CopyFrom(self.dst_uid.into_proto())
        proto_value.forward_edge_name.CopyFrom(self.forward_edge_name.into_proto())
        proto_value.reverse_edge_name.CopyFrom(self.reverse_edge_name.into_proto())
        return proto_value


UpdateInner = (
    StringPropertyUpdate | UInt64PropertyUpdate | Int64PropertyUpdate | EdgeUpdate
)


@dataclasses.dataclass(frozen=True, slots=True)
class Update(SerDe[proto.Update]):
    inner: UpdateInner

    _proto_cls = proto.Update

    @classmethod
    def from_proto(cls, proto_value: proto.Update) -> Update:
        field_name = proto_value.WhichOneof("inner")
        assert field_name is not None

        match field_name:
            case "string_property":
                return cls(
                    inner=StringPropertyUpdate.from_proto(proto_value.string_property)
                )
            case "uint64_property":
                return cls(
                    inner=UInt64PropertyUpdate.from_proto(proto_value.uint64_property)
                )
            case "int64_property":
                return cls(
                    inner=Int64PropertyUpdate.from_proto(proto_value.int64_property)
                )
            case "edge":
                return cls(inner=EdgeUpdate.from_proto(proto_value.edge))

        raise Exception(f"Unknown variant: {field_name}")

    def into_proto(self) -> proto.Update:
        msg = self.new_proto()
        match self.inner:
            case StringPropertyUpdate() as inner:
                msg.string_property.CopyFrom(inner.into_proto())
            case UInt64PropertyUpdate() as inner:
                msg.uint64_property.CopyFrom(inner.into_proto())
            case Int64PropertyUpdate() as inner:
                msg.int64_property.CopyFrom(inner.into_proto())
            case EdgeUpdate() as inner:
                msg.edge.CopyFrom(inner.into_proto())
            case _:
                raise Exception(f"Unknown variant: {self.inner}")

        return msg


@dataclasses.dataclass(frozen=True, slots=True)
class LensRef(SerDe[proto.LensRef]):
    lens_namespace: str
    lens_name: str

    _proto_cls = proto.LensRef

    @classmethod
    def from_proto(
        cls,
        proto_value: proto.LensRef,
    ) -> LensRef:
        return cls(
            lens_namespace=proto_value.lens_namespace,
            lens_name=proto_value.lens_name,
        )

    def into_proto(self) -> proto.LensRef:
        proto_value = self.new_proto()
        proto_value.lens_namespace = self.lens_namespace
        proto_value.lens_name = self.lens_name
        return proto_value


@dataclasses.dataclass(frozen=True, slots=True)
class AnalyzerName(SerDe[proto.AnalyzerName]):
    value: str

    _proto_cls = proto.AnalyzerName

    @classmethod
    def from_proto(
        cls,
        proto_value: proto.AnalyzerName,
    ) -> AnalyzerName:
        return cls(
            value=proto_value.value,
        )

    def into_proto(self) -> proto.AnalyzerName:
        proto_value = self.new_proto()
        proto_value.value = self.value
        return proto_value


@dataclasses.dataclass(frozen=True, slots=True)
class ExecutionHit(SerDe[proto.ExecutionHit]):
    # graph_view: GraphView
    lens_refs: list[LensRef]
    analyzer_name: AnalyzerName
    time_of_match: proto_common_msgs.Timestamp
    idempotency_key: int
    score: int

    _proto_cls = proto.ExecutionHit

    @classmethod
    def from_proto(
        cls,
        proto_value: proto.ExecutionHit,
    ) -> ExecutionHit:
        return cls(
            lens_refs=[LensRef.from_proto(p) for p in proto_value.lens_refs],
            analyzer_name=AnalyzerName.from_proto(proto_value.analyzer_name),
            time_of_match=proto_common_msgs.Timestamp.from_proto(
                proto_value.time_of_match
            ),
            idempotency_key=proto_value.idempotency_key,
            score=proto_value.score,
        )

    def into_proto(self) -> proto.ExecutionHit:
        proto_value = self.new_proto()
        proto_value.lens_refs.extend(
            [lens_ref.into_proto() for lens_ref in self.lens_refs]
        )
        proto_value.analyzer_name.CopyFrom(self.analyzer_name.into_proto())
        proto_value.time_of_match.CopyFrom(self.time_of_match.into_proto())
        proto_value.idempotency_key = self.idempotency_key
        proto_value.score = self.score
        return proto_value


@dataclasses.dataclass(frozen=True, slots=True)
class ExecutionMiss(SerDe[proto.ExecutionMiss]):
    _proto_cls = proto.ExecutionMiss

    @classmethod
    def from_proto(
        cls,
        proto_value: proto.ExecutionMiss,
    ) -> ExecutionMiss:
        return cls()

    def into_proto(self) -> proto.ExecutionMiss:
        proto_value = self.new_proto()
        return proto_value


ExecutionResultInner = ExecutionHit | ExecutionMiss


@dataclasses.dataclass(frozen=True, slots=True)
class ExecutionResult(SerDe[proto.ExecutionResult]):
    inner: ExecutionResultInner

    _proto_cls = proto.ExecutionResult

    @classmethod
    def from_proto(cls, proto_value: proto.ExecutionResult) -> ExecutionResult:
        field_name = proto_value.WhichOneof("inner")
        assert field_name is not None

        match field_name:
            case "hit":
                return cls(inner=ExecutionHit.from_proto(proto_value.hit))
            case "miss":
                return cls(inner=ExecutionMiss.from_proto(proto_value.miss))

        raise Exception(f"Unknown variant: {field_name}")

    def into_proto(self) -> proto.ExecutionResult:
        msg = self.new_proto()
        match self.inner:
            case ExecutionHit() as inner:
                msg.hit.CopyFrom(inner.into_proto())
            case ExecutionMiss() as inner:
                msg.miss.CopyFrom(inner.into_proto())
            case _:
                raise Exception(f"Unknown variant: {self.inner}")

        return msg


@dataclasses.dataclass(frozen=True, slots=True)
class RunAnalyzerResponse(SerDe[proto.RunAnalyzerResponse]):
    execution_result: ExecutionResult

    _proto_cls = proto.RunAnalyzerResponse

    @classmethod
    def from_proto(
        cls,
        proto_value: proto.RunAnalyzerResponse,
    ) -> RunAnalyzerResponse:
        return cls(
            execution_result=ExecutionResult.from_proto(proto_value.execution_result),
        )

    def into_proto(self) -> proto.RunAnalyzerResponse:
        proto_value = self.new_proto()
        proto_value.execution_result.CopyFrom(self.execution_result.into_proto())
        return proto_value
