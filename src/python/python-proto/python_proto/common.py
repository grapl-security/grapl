from __future__ import annotations

import dataclasses
import datetime
import uuid
from typing import Type

from google.protobuf.duration_pb2 import Duration as _Duration
from google.protobuf.timestamp_pb2 import Timestamp as _Timestamp
from graplinc.common.v1beta1.types_pb2 import Uuid as _Uuid
from python_proto import SerDe


SECONDS_PER_DAY = 60 * 60 * 24
EPOCH = datetime.datetime.fromisoformat("1970-01-01T00:00:00.000")


@dataclasses.dataclass(frozen=True)
class Uuid(SerDe[_Uuid]):
    lsb: int
    msb: int
    proto_cls: Type[_Uuid] = _Uuid

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
class Timestamp(SerDe[_Timestamp]):
    seconds: int
    nanos: int
    proto_cls: Type[_Timestamp] = _Timestamp

    @staticmethod
    def deserialize(bytes_: bytes) -> SerDe[_Timestamp]:
        proto_timestamp = _Timestamp()
        proto_timestamp.ParseFromString(bytes_)
        return Timestamp.from_proto(proto_timestamp=proto_timestamp)

    @staticmethod
    def from_datetime(datetime_: datetime.datetime) -> Timestamp:
        duration = datetime_ - EPOCH
        return Timestamp(
            seconds=duration.days * SECONDS_PER_DAY + duration.seconds,
            nanos=duration.microseconds * 1000,
        )

    def into_datetime(self) -> datetime.datetime:
        return EPOCH + datetime.timedelta(
            seconds=self.seconds,
            microseconds=self.nanos // 1000
        )

    @staticmethod
    def from_proto(proto_timestamp: _Timestamp) -> Timestamp:
        return Timestamp(seconds=proto_timestamp.seconds, nanos=proto_timestamp.nanos)

    def into_proto(self) -> _Timestamp:
        proto_timestamp = _Timestamp()
        proto_timestamp.seconds = self.seconds
        proto_timestamp.nanos = self.nanos
        return proto_timestamp


@dataclasses.dataclass(frozen=True)
class Duration(SerDe[_Duration]):
    seconds: int
    nanos: int
    proto_cls: Type[_Duration] = _Duration

    @staticmethod
    def deserialize(bytes_: bytes) -> SerDe[_Duration]:
        proto_duration = _Duration()
        proto_duration.ParseFromString(bytes_)
        return Duration.from_proto(proto_duration=proto_duration)

    @staticmethod
    def from_timedelta(timedelta: datetime.timedelta) -> Duration:
        return Duration(
            seconds=timedelta.days * SECONDS_PER_DAY + timedelta.seconds,
            nanos=timedelta.microseconds * 1000
        )

    def into_timedelta(self) -> datetime.timedelta:
        return datetime.timedelta(seconds=self.seconds, microseconds=self.nanos // 1000)

    @staticmethod
    def from_proto(proto_duration: _Duration) -> SerDe[_Duration]:
        return Duration(
            seconds=proto_duration.seconds,
            nanos=proto_duration.nanos,
        )

    def into_proto(self) -> _Duration:
        proto_duration = _Duration()
        proto_duration.seconds = self.seconds
        proto_duration.nanos = self.nanos
        return proto_duration
