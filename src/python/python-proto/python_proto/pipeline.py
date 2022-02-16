from __future__ import annotations

import dataclasses
import datetime
import uuid
from typing import Generic, Type

from google.protobuf.any_pb2 import Any as _Any
from graplinc.grapl.pipeline.v1beta1.types_pb2 import Envelope as _Envelope
from graplinc.grapl.pipeline.v1beta1.types_pb2 import Metadata as _Metadata
from graplinc.grapl.pipeline.v1beta1.types_pb2 import RawLog as _RawLog
from python_proto import I, SerDe, SerDeWithInner
from python_proto.common import Timestamp, Uuid


@dataclasses.dataclass(frozen=True)
class Metadata(SerDe[_Metadata]):
    trace_id: uuid.UUID
    tenant_id: uuid.UUID
    event_source_id: uuid.UUID
    created_time: datetime.datetime
    last_updated_time: datetime.datetime
    proto_cls: Type[_Metadata] = _Metadata

    @staticmethod
    def deserialize(bytes_: bytes) -> Metadata:
        proto_metadata = _Metadata()
        proto_metadata.ParseFromString(bytes_)
        return Metadata.from_proto(proto_metadata=proto_metadata)

    @staticmethod
    def from_proto(proto_metadata: _Metadata) -> Metadata:
        return Metadata(
            trace_id=Uuid.from_proto(proto_metadata.trace_id).into_uuid(),
            tenant_id=Uuid.from_proto(proto_metadata.tenant_id).into_uuid(),
            event_source_id=Uuid.from_proto(proto_metadata.event_source_id).into_uuid(),
            created_time=Timestamp.from_proto(
                proto_metadata.created_time
            ).into_datetime(),
            last_updated_time=Timestamp.from_proto(
                proto_metadata.last_updated_time
            ).into_datetime(),
        )

    def into_proto(self) -> _Metadata:
        proto_metadata = _Metadata()
        proto_metadata.trace_id.CopyFrom(Uuid.from_uuid(self.trace_id).into_proto())
        proto_metadata.tenant_id.CopyFrom(Uuid.from_uuid(self.tenant_id).into_proto())
        proto_metadata.event_source_id.CopyFrom(
            Uuid.from_uuid(self.event_source_id).into_proto()
        )
        proto_metadata.created_time.CopyFrom(
            Timestamp.from_datetime(self.created_time).into_proto()
        )
        proto_metadata.last_updated_time.CopyFrom(
            Timestamp.from_datetime(self.last_updated_time).into_proto()
        )
        return proto_metadata


@dataclasses.dataclass(frozen=True)
class Envelope(SerDeWithInner[_Envelope, I], Generic[I]):
    metadata: Metadata
    inner_message: I
    proto_cls: Type[_Envelope] = _Envelope

    @staticmethod
    def deserialize(bytes_: bytes, inner_cls: Type[I]) -> Envelope[I]:
        proto_envelope = _Envelope()
        proto_envelope.ParseFromString(bytes_)
        return Envelope.from_proto(proto_envelope=proto_envelope, inner_cls=inner_cls)

    @staticmethod
    def from_proto(proto_envelope: _Envelope, inner_cls: Type[I]) -> Envelope[I]:
        inner_message = inner_cls.proto_cls()
        proto_envelope.inner_message.Unpack(inner_message)
        return Envelope(
            metadata=Metadata.from_proto(proto_envelope.metadata),
            inner_message=inner_cls.from_proto(inner_message),
        )

    def into_proto(self) -> _Envelope:
        proto_envelope = _Envelope()
        proto_envelope.metadata.CopyFrom(self.metadata.into_proto())
        inner_message = _Any()
        inner_message.Pack(
            self.inner_message.into_proto(), type_url_prefix="".encode("utf-8")
        )
        proto_envelope.inner_message.CopyFrom(inner_message)
        return proto_envelope


@dataclasses.dataclass(frozen=True)
class RawLog(SerDe[_RawLog]):  # FIXME: test
    log_event: bytes
    proto_cls: Type[_RawLog] = _RawLog

    @staticmethod
    def deserialize(bytes_: bytes) -> RawLog:
        proto_raw_log = _RawLog()
        proto_raw_log.ParseFromString(bytes_)
        return RawLog.from_proto(proto_raw_log=proto_raw_log)

    @staticmethod
    def from_proto(proto_raw_log: _RawLog) -> RawLog:
        return RawLog(log_event=proto_raw_log.log_event)

    def into_proto(self) -> _RawLog:
        proto_raw_log = _RawLog()
        proto_raw_log.log_event = self.log_event
        return proto_raw_log
