from __future__ import annotations

import dataclasses
import datetime
import uuid
from typing import Type

from graplinc.common.v1beta1.types_pb2 import Duration as _Duration
from graplinc.common.v1beta1.types_pb2 import Timestamp as _Timestamp
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
class Duration(SerDe[_Duration]):
    seconds: int
    nanos: int
    proto_cls: Type[_Duration] = _Duration

    def __post_init__(self) -> None:
        if self.seconds < 0 or self.nanos < 0:
            raise TypeError(
                f"Duration cannot be negative. Received seconds: {self.seconds} nanos: {self.nanos}"
            )

    @staticmethod
    def deserialize(bytes_: bytes) -> SerDe[_Duration]:
        proto_duration = _Duration()
        proto_duration.ParseFromString(bytes_)
        return Duration.from_proto(proto_duration=proto_duration)

    @staticmethod
    def from_timedelta(timedelta: datetime.timedelta) -> Duration:
        if timedelta.days < 0 or timedelta.seconds < 0 or timedelta.microseconds < 0:
            raise ValueError(
                f"Durations must be positive. Encountered days: {timedelta.days} seconds: {timedelta.seconds} microseconds: {timedelta.microseconds}"
            )
        return Duration(
            seconds=timedelta.days * SECONDS_PER_DAY + timedelta.seconds,
            nanos=timedelta.microseconds * 1000,
        )

    def into_timedelta(self) -> datetime.timedelta:
        """Note that python's timedelta only offers microsecond precision"""
        return datetime.timedelta(seconds=self.seconds, microseconds=self.nanos // 1000)

    @staticmethod
    def from_proto(proto_duration: _Duration) -> Duration:
        return Duration(
            seconds=proto_duration.seconds,
            nanos=proto_duration.nanos,
        )

    def into_proto(self) -> _Duration:
        proto_duration = _Duration()
        proto_duration.seconds = self.seconds
        proto_duration.nanos = self.nanos
        return proto_duration


@dataclasses.dataclass(frozen=True)
class Timestamp(SerDe[_Timestamp]):
    duration: Duration
    before_epoch: bool
    proto_cls: Type[_Timestamp] = _Timestamp

    @staticmethod
    def deserialize(bytes_: bytes) -> SerDe[_Timestamp]:
        proto_timestamp = _Timestamp()
        proto_timestamp.ParseFromString(bytes_)
        return Timestamp.from_proto(proto_timestamp=proto_timestamp)

    @staticmethod
    def from_datetime(datetime_: datetime.datetime) -> Timestamp:
        if datetime_ < EPOCH:
            timedelta = EPOCH - datetime_
            return Timestamp(
                duration=Duration(
                    seconds=timedelta.days * SECONDS_PER_DAY + timedelta.seconds,
                    nanos=timedelta.microseconds * 1000,
                ),
                before_epoch=True,
            )
        else:
            timedelta = datetime_ - EPOCH
            return Timestamp(
                duration=Duration(
                    seconds=timedelta.days * SECONDS_PER_DAY + timedelta.seconds,
                    nanos=timedelta.microseconds * 1000,
                ),
                before_epoch=False,
            )

    def into_datetime(self) -> datetime.datetime:
        """Note that python's datetime only offers microsecond precision"""
        duration = datetime.timedelta(
            seconds=self.duration.seconds, microseconds=self.duration.nanos // 1000
        )
        if self.before_epoch:
            return EPOCH - duration
        else:
            return EPOCH + duration

    @staticmethod
    def from_proto(proto_timestamp: _Timestamp) -> Timestamp:
        assert proto_timestamp.WhichOneof("duration") is not None
        if proto_timestamp.WhichOneof("duration") == "since_epoch":
            proto_duration = proto_timestamp.since_epoch
            return Timestamp(
                duration=Duration.from_proto(proto_duration),
                before_epoch=False,
            )
        elif proto_timestamp.WhichOneof("duration") == "before_epoch":
            proto_duration = proto_timestamp.before_epoch
            return Timestamp(
                duration=Duration.from_proto(proto_duration), before_epoch=True
            )
        else:
            raise ValueError("proto_timestamp contains invalid duration")

    def into_proto(self) -> _Timestamp:
        proto_timestamp = _Timestamp()
        if self.before_epoch:
            proto_timestamp.before_epoch.CopyFrom(self.duration.into_proto())
        else:
            proto_timestamp.since_epoch.CopyFrom(self.duration.into_proto())
        return proto_timestamp
