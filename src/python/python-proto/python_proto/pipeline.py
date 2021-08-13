from __future__ import annotations

import json
import uuid

from typing import cast

from graplinc.grapl.pipeline.v1beta1.types_pb2 import Envelope as _Envelope
from graplinc.grapl.pipeline.v1beta1.types_pb2 import Metadata as _Metadata
from graplinc.grapl.pipeline.v1beta1.types_pb2 import Uuid as _Uuid

from python_proto import SerDe

class Uuid(SerDe):
    lsb: int
    msb: int

    def __init__(self, lsb: int, msb: int) -> None:
        self.lsb = lsb
        self.msb = msb

    @staticmethod
    def deserialize(bytes_: bytes) -> Uuid:
        uuid_ = _Uuid()
        uuid_.parseFromString(bytes_)
        return Uuid._from_proto(proto_uuid=uuid_)

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

    def serialize(self) -> bytes:
        return cast(bytes, self._into_proto().SerializeToString())

    @staticmethod
    def _from_proto(proto_uuid: _Uuid) -> Uuid:
        return Uuid(lsb=proto_uuid.lsb, msb=proto_uuid.msb)

    def _into_proto(self) -> _Uuid:
        proto_uuid = _Uuid()
        proto_uuid.lsb = self.lsb
        proto_uuid.msb = self.msb
        return proto_uuid

    def __str__(self) -> str:
        return str(self.into_uuid())

    def __repr__(self) -> str:
        return "Uuid " + json.dumps({"lsb": self.lsb, "msb": self.msb})

    def __hash__(self) -> int:
        return hash(self.into_uuid())

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, Uuid):
            return NotImplemented
        return self.into_uuid() == other.into_uuid()


class Metadata(SerDe):
    trace_id: uuid.UUID
    tenant_id: uuid.UUID

    def __init__(self, trace_id: uuid.UUID, tenant_id: uuid.UUID) -> None:
        self.trace_id = trace_id
        self.tenant_id = tenant_id

    @staticmethod
    def deserialize(bytes_: bytes) -> Metadata:
        metadata = _Metadata()
        metadata.ParseFromString(bytes_)
        return Metadata._from_proto(metadata)

    def serialize(self) -> bytes:
        return cast(bytes, self._into_proto().SerializeToString())

    @staticmethod
    def _from_proto_parts(trace_id: _Uuid, tenant_id: _Uuid) -> Metadata:
        return Metadata(
            trace_id=Uuid._from_proto(trace_id).into_uuid(),
            tenant_id=Uuid._from_proto(tenant_id).into_uuid(),
        )

    @staticmethod
    def _from_proto(metadata: _Metadata) -> Metadata:
        return Metadata._from_proto_parts(metadata.trace_id, metadata.tenant_id)

    def _into_proto(self) -> _Metadata:
        metadata = _Metadata()
        metadata.trace_id.CopyFrom(Uuid.from_uuid(self.trace_id)._into_proto())
        metadata.tenant_id.CopyFrom(Uuid.from_uuid(self.tenant_id)._into_proto())
        return metadata

    def __str__(self) -> str:
        return repr(self)

    def __repr__(self) -> str:
        return "Metadata " + json.dumps(
            {
                "trace_id": str(self.trace_id),
                "tenant_id": str(self.tenant_id),
            }
        )

    def __hash__(self) -> int:
        return hash((self.trace_id, self.tenant_id))

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, Metadata):
            return NotImplemented
        return (
            self.trace_id == other.trace_id
            and self.tenant_id == other.tenant_id
        )


class Envelope(SerDe):
    metadata: Metadata
    inner_message: bytes
    inner_type: str

    def __init__(
        self,
        metadata: Metadata,
        inner_message: bytes,
        inner_type: str,
    ) -> None:
        self.metadata = metadata
        self.inner_message = inner_message
        self.inner_type = inner_type

    @staticmethod
    def deserialize(bytes_: bytes) -> Envelope:
        envelope = _Envelope()
        envelope.ParseFromString(bytes_)
        return Envelope._from_proto(envelope)

    def serialize(self) -> bytes:
        return cast(bytes, self._into_proto().SerializeToString())

    @staticmethod
    def _from_proto(envelope: _Envelope) -> Envelope:
        return Envelope._from_proto_parts(
            envelope.metadata,
            envelope.inner_message,
            envelope.inner_type,
        )

    @staticmethod
    def _from_proto_parts(
        metadata: _Metadata,
        inner_message: bytes,
        inner_type: str,
    ) -> Envelope:
        return Envelope(Metadata._from_proto(metadata), inner_message, inner_type)

    def _into_proto(self) -> _Envelope:
        envelope = _Envelope()
        envelope.metadata.CopyFrom(self.metadata._into_proto())
        envelope.inner_message = self.inner_message
        envelope.inner_type = self.inner_type
        return envelope

    def __str__(self) -> str:
        return repr(self)

    def __repr__(self) -> str:
        return "Envelope " + json.dumps(
            {
                "metadata": repr(self.metadata),
                "inner_type": self.inner_type,
                # The message isn't valid utf8, and it could be huge, so just print out its length
                "inner_message_len": len(self.inner_message),
            }
        )

    def __hash__(self) -> int:
        return hash((self.metadata, self.inner_message, self.inner_type))

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, Envelope):
            return NotImplemented
        return (
            self.metadata == other.metadata
            and self.inner_message == other.inner_message
            and self.inner_type == other.inner_type
        )
