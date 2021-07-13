from __future__ import annotations

import uuid
from uuid import uuid4

from graplinc.grapl.api.services.v1beta1.types_pb2 import Envelope as _Envelope
from graplinc.grapl.api.services.v1beta1.types_pb2 import Metadata, ProtoUuidV4


def proto_uuid_to_pyuuid(proto_uuid: ProtoUuidV4) -> uuid.UUID:
    lsb = proto_uuid.lsb
    msb = proto_uuid.msb
    lsb_bytes = int.to_bytes(lsb, byteorder="little", length=8, signed=False)
    msb_bytes = int.to_bytes(msb, byteorder="little", length=8, signed=False)

    return uuid.UUID(bytes=lsb_bytes + msb_bytes)


def pyuuid_to_proto_uuid(pyuuid: uuid.UUID) -> ProtoUuidV4:
    lsb_bytes = pyuuid.bytes[:8]
    msb_bytes = pyuuid.bytes[8:]
    lsb = int.from_bytes(lsb_bytes, byteorder="little", signed=False)
    msb = int.from_bytes(msb_bytes, byteorder="little", signed=False)

    proto_uuid = ProtoUuidV4()
    proto_uuid.lsb = lsb
    proto_uuid.msb = msb
    return proto_uuid


def init_metadata() -> Metadata:
    metadata = Metadata()
    metadata.trace_id.CopyFrom(pyuuid_to_proto_uuid(uuid4()))
    metadata.tenant_id.CopyFrom(pyuuid_to_proto_uuid(uuid4()))
    return metadata


class Envelope(object):
    def __init__(self, envelope: _Envelope) -> None:
        self.envelope = envelope

    @staticmethod
    def from_parts(metadata: Metadata, inner_message: bytes, inner_type: str) -> Envelope:
        envelope = _Envelope()
        envelope.metadata.CopyFrom(metadata)
        envelope.inner_message = inner_message
        envelope.inner_type = inner_type
        return Envelope(envelope)

    @staticmethod
    def from_proto(s: bytes) -> Envelope:
        envelope = _Envelope()
        envelope.ParseFromString(s)

        return Envelope(envelope)

    def serialize_to_string(self) -> str:
        return self.envelope.SerializeToString()

    @property
    def metadata(self) -> Metadata:
        return self.envelope.metadata

    @property
    def inner_type(self) -> str:
        return self.envelope.inner_type

    @property
    def inner_message(self) -> bytes:
        return self.envelope.inner_message
