from __future__ import annotations

import dataclasses
import uuid

from graplinc.grapl.pipeline.v1beta1.types_pb2 import Envelope as _Envelope
from graplinc.grapl.pipeline.v1beta1.types_pb2 import Metadata as _Metadata
from graplinc.grapl.pipeline.v1beta1.types_pb2 import Uuid as _Uuid
from python_proto import SerDe


@dataclasses.dataclass(frozen=True)
class Uuid(SerDe):
    lsb: int
    msb: int

    @staticmethod
    def deserialize(bytes_: bytes) -> Uuid:
        proto_uuid = _Uuid()
        proto_uuid.ParseFromString(bytes_)
        return Uuid.from_proto(proto_uuid=proto_uuid)

    @staticmethod
    def from_uuid(uuid_: uuid.UUID) -> Uuid:
        lsb_bytes = uuid_.bytes[:8]
        msb_bytes = uuid_.bytes[8:]
        lsb = int.from_bytes(lsb_bytes, byteorder="little", signed=False)
        msb = int.from_bytes(msb_bytes, byteorder="little", signed=False)
        return Uuid(lsb=lsb, msb=msb)

    def into_uuid(self) -> uuid.UUID:
        lsb_bytes = int.to_bytes(self.lsb, byteorder="little", length=8, signed=False)
        msb_bytes = int.to_bytes(self.msb, byteorder="little", length=8, signed=False)
        return uuid.UUID(bytes=lsb_bytes + msb_bytes)

    @staticmethod
    def from_proto(proto_uuid: _Uuid) -> Uuid:
        return Uuid(lsb=proto_uuid.lsb, msb=proto_uuid.msb)

    def into_proto(self) -> _Uuid:
        proto_uuid = _Uuid()
        proto_uuid.lsb = self.lsb
        proto_uuid.msb = self.msb
        return proto_uuid


@dataclasses.dataclass(frozen=True)
class Metadata(SerDe):
    trace_id: uuid.UUID
    tenant_id: uuid.UUID

    @staticmethod
    def deserialize(bytes_: bytes) -> Metadata:
        proto_metadata = _Metadata()
        proto_metadata.ParseFromString(bytes_)
        return Metadata.from_proto(proto_metadata=proto_metadata)

    @staticmethod
    def _from_proto_parts(trace_id: _Uuid, tenant_id: _Uuid) -> Metadata:
        return Metadata(
            trace_id=Uuid.from_proto(trace_id).into_uuid(),
            tenant_id=Uuid.from_proto(tenant_id).into_uuid(),
        )

    @staticmethod
    def from_proto(proto_metadata: _Metadata) -> Metadata:
        return Metadata._from_proto_parts(
            proto_metadata.trace_id, proto_metadata.tenant_id
        )

    def into_proto(self) -> _Metadata:
        proto_metadata = _Metadata()
        proto_metadata.trace_id.CopyFrom(Uuid.from_uuid(self.trace_id).into_proto())
        proto_metadata.tenant_id.CopyFrom(Uuid.from_uuid(self.tenant_id).into_proto())
        return proto_metadata


@dataclasses.dataclass(frozen=True)
class Envelope(SerDe):
    metadata: Metadata
    inner_message: bytes
    inner_type: str

    @staticmethod
    def deserialize(bytes_: bytes) -> Envelope:
        proto_envelope = _Envelope()
        proto_envelope.ParseFromString(bytes_)
        return Envelope.from_proto(proto_envelope=proto_envelope)

    @staticmethod
    def from_proto(proto_envelope: _Envelope) -> Envelope:
        return Envelope._from_proto_parts(
            proto_envelope.metadata,
            proto_envelope.inner_message,
            proto_envelope.inner_type,
        )

    @staticmethod
    def _from_proto_parts(
        metadata: _Metadata,
        inner_message: bytes,
        inner_type: str,
    ) -> Envelope:
        return Envelope(Metadata.from_proto(metadata), inner_message, inner_type)

    def into_proto(self) -> _Envelope:
        proto_envelope = _Envelope()
        proto_envelope.metadata.CopyFrom(self.metadata.into_proto())
        proto_envelope.inner_message = self.inner_message
        proto_envelope.inner_type = self.inner_type
        return proto_envelope
