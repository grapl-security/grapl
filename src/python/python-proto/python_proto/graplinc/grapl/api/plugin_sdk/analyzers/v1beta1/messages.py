from __future__ import annotations

#
# import abc
# import datetime
# import uuid
# from dataclasses import dataclass
# from typing import Sequence, Optional, cast, Union
from dataclasses import InitVar, dataclass, field
from datetime import datetime
from types import NoneType
from typing import List, Union, final

from graplinc.grapl.api.plugin_sdk.analyzers.v1beta1.analyzers_pb2 import (
    AnalyzerName as AnalyzerNameProto,
)
from graplinc.grapl.api.plugin_sdk.analyzers.v1beta1.analyzers_pb2 import (
    ExecutionHit as ExecutionHitProto,
)
from graplinc.grapl.api.plugin_sdk.analyzers.v1beta1.analyzers_pb2 import (
    ExecutionMiss as ExecutionMissProto,
)
from graplinc.grapl.api.plugin_sdk.analyzers.v1beta1.analyzers_pb2 import (
    ExecutionResult as ExecutionResultProto,
)
from graplinc.grapl.api.plugin_sdk.analyzers.v1beta1.analyzers_pb2 import (
    Int64PropertyUpdate as Int64PropertyUpdateProto,
)
from graplinc.grapl.api.plugin_sdk.analyzers.v1beta1.analyzers_pb2 import (
    LensRef as LensRefProto,
)
from graplinc.grapl.api.plugin_sdk.analyzers.v1beta1.analyzers_pb2 import (
    RunAnalyzerRequest as RunAnalyzerRequestProto,
)
from graplinc.grapl.api.plugin_sdk.analyzers.v1beta1.analyzers_pb2 import (
    RunAnalyzerResponse as RunAnalyzerResponseProto,
)
from graplinc.grapl.api.plugin_sdk.analyzers.v1beta1.analyzers_pb2 import (
    StringPropertyUpdate as StringPropertyUpdateProto,
)
from graplinc.grapl.api.plugin_sdk.analyzers.v1beta1.analyzers_pb2 import (
    UInt64PropertyUpdate as UInt64PropertyUpdateProto,
)
from graplinc.grapl.api.plugin_sdk.analyzers.v1beta1.analyzers_pb2 import (
    Update as UpdateProto,
)
from python_proto import SerDe
from python_proto.common import Timestamp, Uuid
from python_proto.graplinc.grapl.api.graph_query.v1beta1.messages import (
    GraphView,
    PropertyName,
    Uid,
)
from typing_extensions import Never, assert_never


@dataclass(frozen=True, slots=True)
class StringPropertyUpdate(SerDe[StringPropertyUpdateProto]):
    uid: Uid
    property_name: PropertyName

    @staticmethod
    def deserialize(bytes_: bytes) -> StringPropertyUpdate:
        msg = StringPropertyUpdateProto()
        return StringPropertyUpdate.from_proto(msg.FromString(bytes_))

    @staticmethod
    def from_proto(
        proto: StringPropertyUpdateProto,
    ) -> StringPropertyUpdate:
        return StringPropertyUpdate(
            uid=Uid.from_proto(proto.uid),
            property_name=PropertyName.from_proto(proto.property_name),
        )

    def into_proto(self) -> StringPropertyUpdateProto:
        msg = StringPropertyUpdateProto()
        msg.uid.CopyFrom(self.uid.into_proto())
        msg.property_name.CopyFrom(self.property_name.into_proto())
        return msg


@dataclass(frozen=True, slots=True)
class UInt64PropertyUpdate(SerDe[UInt64PropertyUpdateProto]):
    uid: Uid
    property_name: PropertyName

    @staticmethod
    def deserialize(bytes_: bytes) -> UInt64PropertyUpdate:
        msg = UInt64PropertyUpdateProto()
        return UInt64PropertyUpdate.from_proto(msg.FromString(bytes_))

    @staticmethod
    def from_proto(
        proto: UInt64PropertyUpdateProto,
    ) -> UInt64PropertyUpdate:
        return UInt64PropertyUpdate(
            uid=Uid.from_proto(proto.uid),
            property_name=PropertyName.from_proto(proto.property_name),
        )

    def into_proto(self) -> UInt64PropertyUpdateProto:
        msg = UInt64PropertyUpdateProto()
        msg.uid.CopyFrom(self.uid.into_proto())
        msg.property_name.CopyFrom(self.property_name.into_proto())
        return msg


@dataclass(frozen=True, slots=True)
class Int64PropertyUpdate(SerDe[Int64PropertyUpdateProto]):
    uid: Uid
    property_name: PropertyName

    @staticmethod
    def deserialize(bytes_: bytes) -> Int64PropertyUpdate:
        msg = Int64PropertyUpdateProto()
        return Int64PropertyUpdate.from_proto(msg.FromString(bytes_))

    @staticmethod
    def from_proto(
        proto: Int64PropertyUpdateProto,
    ) -> Int64PropertyUpdate:
        return Int64PropertyUpdate(
            uid=Uid.from_proto(proto.uid),
            property_name=PropertyName.from_proto(proto.property_name),
        )

    def into_proto(self) -> Int64PropertyUpdateProto:
        msg = Int64PropertyUpdateProto()
        msg.uid.CopyFrom(self.uid.into_proto())
        msg.property_name.CopyFrom(self.property_name.into_proto())
        return msg


@dataclass(frozen=True, slots=True)
class Update(SerDe[UpdateProto]):
    inner: Union[
        StringPropertyUpdate,
        UInt64PropertyUpdate,
        Int64PropertyUpdate,
    ]

    @staticmethod
    def deserialize(bytes_: bytes) -> Update:
        msg = UpdateProto()
        return Update.from_proto(msg.FromString(bytes_))

    @staticmethod
    def from_proto(
        proto: UpdateProto,
    ) -> Update:
        variant = proto.WhichOneof("inner")
        inner: Union[
            StringPropertyUpdate,
            UInt64PropertyUpdate,
            Int64PropertyUpdate,
        ]

        match variant:
            case "string_property_update":
                if string_property_update := proto.string_property_update:
                    inner = StringPropertyUpdate.from_proto(string_property_update)
                else:
                    raise ValueError(
                        "Invalid proto, variant is string_property_update but field is None"
                    )
            case "uint64_property_update":
                if uint64_property_update := proto.uint64_property_update:
                    inner = UInt64PropertyUpdate.from_proto(uint64_property_update)
                else:
                    raise ValueError(
                        "Invalid proto, variant is uint64_property_update but field is None"
                    )
            case "int64_property_update":
                if int64_property_update := proto.int64_property_update:
                    inner = Int64PropertyUpdate.from_proto(int64_property_update)
                else:
                    raise ValueError(
                        "Invalid proto, variant is int64_property_update but field is None"
                    )
            case None:
                raise Exception("Update.inner was None")
            case _:
                assert_never(variant)

        return Update(
            inner=inner,
        )

    def into_proto(self) -> UpdateProto:
        msg = UpdateProto()
        match self.inner:
            case StringPropertyUpdate() as inner:
                msg.string_property_update.CopyFrom(inner.into_proto())
            case UInt64PropertyUpdate() as inner:
                msg.uint64_property_update.CopyFrom(inner.into_proto())
            case Int64PropertyUpdate() as inner:
                msg.int64_property_update.CopyFrom(inner.into_proto())
            case _:
                assert_never(self.inner)

        return msg


@dataclass(frozen=True, slots=True)
@final
class AnalyzerName(SerDe[AnalyzerNameProto]):
    value: str  # todo: Replace with LiteralString when Python 3.11 is stable
    _skip_check: InitVar[bool] = field(default=False)

    def __post_init__(self, _skip_check: bool) -> None:
        if _skip_check:
            return
        # Check if valid
        if not self.value:
            raise Exception("AnalyerName must not be empty")
        if len(self.value) > 48:
            raise Exception("AnalyerName must not be longer than 48 characters")

    @staticmethod
    def deserialize(bytes_: bytes) -> AnalyzerName:
        msg = AnalyzerNameProto()
        return AnalyzerName.from_proto(msg.FromString(bytes_))

    @staticmethod
    def from_proto(
        proto: AnalyzerNameProto,
    ) -> AnalyzerName:
        return AnalyzerName(
            value=proto.value,
        )

    def into_proto(self) -> AnalyzerNameProto:
        msg = AnalyzerNameProto()
        msg.value = self.value
        return msg


@dataclass(frozen=True, slots=True)
class LensRef(SerDe[LensRefProto]):
    lens_namespace: str
    lens_name: str

    @staticmethod
    def deserialize(bytes_: bytes) -> LensRef:
        msg = LensRefProto()
        return LensRef.from_proto(msg.FromString(bytes_))

    @staticmethod
    def from_proto(
        proto: LensRefProto,
    ) -> LensRef:
        return LensRef(
            lens_namespace=proto.lens_namespace,
            lens_name=proto.lens_name,
        )

    def into_proto(self) -> LensRefProto:
        msg = LensRefProto()
        msg.lens_namespace = self.lens_namespace
        msg.lens_name = self.lens_name
        return msg


@dataclass(frozen=True, slots=True)
class ExecutionHit(SerDe[ExecutionHitProto]):
    graph_view: GraphView
    root_uid: Uid
    lens_refs: List[LensRef]
    analyzer_name: AnalyzerName
    idempotency_key: int
    score: int
    _time_of_match: Timestamp

    @staticmethod
    def deserialize(bytes_: bytes) -> ExecutionHit:
        msg = ExecutionHitProto()
        return ExecutionHit.from_proto(msg.FromString(bytes_))

    @staticmethod
    def from_proto(
        proto: ExecutionHitProto,
    ) -> ExecutionHit:
        return ExecutionHit(
            graph_view=GraphView.from_proto(proto.graph_view),
            root_uid=Uid.from_proto(proto.root_uid),
            lens_refs=list(LensRef.from_proto(l) for l in proto.lens_refs),
            analyzer_name=AnalyzerName.from_proto(proto.analyzer_name),
            idempotency_key=proto.idempotency_key,
            score=proto.score,
            _time_of_match=Timestamp.from_proto(proto.time_of_match),
        )

    def into_proto(self) -> ExecutionHitProto:
        msg = ExecutionHitProto()
        msg.graph_view.CopyFrom(self.graph_view.into_proto())
        msg.root_uid.CopyFrom(self.root_uid.into_proto())
        msg.lens_refs.extend(l.into_proto() for l in self.lens_refs)
        msg.analyzer_name.CopyFrom(self.analyzer_name.into_proto())
        msg.idempotency_key = self.idempotency_key
        msg.score = self.score
        msg.time_of_match.CopyFrom(self._time_of_match.into_proto())

        return msg


@dataclass(frozen=True, slots=True)
class ExecutionMiss(SerDe[ExecutionMissProto]):
    @staticmethod
    def deserialize(bytes_: bytes) -> ExecutionMiss:
        msg = ExecutionMissProto()
        return ExecutionMiss.from_proto(msg.FromString(bytes_))

    @staticmethod
    def from_proto(
        proto: ExecutionMissProto,
    ) -> ExecutionMiss:
        return ExecutionMiss()

    def into_proto(self) -> ExecutionMissProto:
        msg = ExecutionMissProto()
        return msg


@dataclass(frozen=True, slots=True)
class RunAnalyzerRequest(SerDe[RunAnalyzerRequestProto]):
    tenant_id: Uuid
    update: Update

    @staticmethod
    def deserialize(bytes_: bytes) -> RunAnalyzerRequest:
        msg = RunAnalyzerRequestProto()
        return RunAnalyzerRequest.from_proto(msg.FromString(bytes_))

    @staticmethod
    def from_proto(
        proto: RunAnalyzerRequestProto,
    ) -> RunAnalyzerRequest:
        return RunAnalyzerRequest(
            tenant_id=Uuid.from_proto(proto.tenant_id),
            update=Update.from_proto(proto.update),
        )

    def into_proto(self) -> RunAnalyzerRequestProto:
        msg = RunAnalyzerRequestProto()
        msg.tenant_id.CopyFrom(self.tenant_id.into_proto())
        msg.update.CopyFrom(self.update.into_proto())
        return msg


@dataclass(frozen=True, slots=True)
class ExecutionResult(SerDe[ExecutionResultProto]):
    inner: Union[
        ExecutionHit,
        ExecutionMiss,
    ]

    @staticmethod
    def miss() -> ExecutionResult:
        return ExecutionResult(
            inner=ExecutionMiss(),
        )

    @staticmethod
    def deserialize(bytes_: bytes) -> ExecutionResult:
        msg = ExecutionResultProto()
        return ExecutionResult.from_proto(msg.FromString(bytes_))

    @staticmethod
    def from_proto(
        proto: ExecutionResultProto,
    ) -> ExecutionResult:
        variant = proto.WhichOneof("inner")
        inner: Union[
            ExecutionHit,
            ExecutionMiss,
        ]

        match variant:
            case "hit":
                if hit := proto.hit:
                    inner = ExecutionHit.from_proto(hit)
                else:
                    raise ValueError("Invalid proto, variant is hit but field is None")
            case "miss":
                if miss := proto.miss:
                    inner = ExecutionMiss.from_proto(miss)
                else:
                    raise ValueError("Invalid proto, variant is miss but field is None")
            case None:
                raise Exception("Update.inner was None")
            case _:
                assert_never(variant)

        return ExecutionResult(
            inner=inner,
        )

    def into_proto(self) -> ExecutionResultProto:
        msg = ExecutionResultProto()
        match self.inner:
            case ExecutionHit() as inner:
                msg.hit.CopyFrom(inner.into_proto())
            case ExecutionMiss() as inner:
                msg.miss.CopyFrom(inner.into_proto())
            case _:
                assert_never(self.inner)

        return msg


@dataclass(frozen=True, slots=True)
class RunAnalyzerResponse(SerDe[RunAnalyzerResponseProto]):
    execution_result: ExecutionResult

    @staticmethod
    def miss() -> RunAnalyzerResponse:
        return RunAnalyzerResponse(execution_result=ExecutionResult.miss())

    @staticmethod
    def deserialize(bytes_: bytes) -> RunAnalyzerResponse:
        msg = RunAnalyzerResponseProto()
        return RunAnalyzerResponse.from_proto(msg.FromString(bytes_))

    @staticmethod
    def from_proto(
        proto: RunAnalyzerResponseProto,
    ) -> RunAnalyzerResponse:
        return RunAnalyzerResponse(
            execution_result=ExecutionResult.from_proto(proto.execution_result)
        )

    def into_proto(self) -> RunAnalyzerResponseProto:
        msg = RunAnalyzerResponseProto()
        msg.execution_result.CopyFrom(self.execution_result.into_proto())
        return msg
