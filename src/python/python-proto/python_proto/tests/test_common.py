import datetime
import uuid

import pytest

pytest.register_assert_rewrite("python_proto.tests.helpers")

from hypothesis import given
from hypothesis import strategies as st
from python_proto.common import Duration, Timestamp, Uuid
from python_proto.tests.helpers import check_encode_decode_invariant
from python_proto.tests.strategies import durations, timestamps, uuids


def test_uuid_encode_decode() -> None:
    check_encode_decode_invariant(uuids())


@given(st.uuids())
def test_uuid_from_into(uuid_: uuid.UUID) -> None:
    assert Uuid.from_uuid(uuid_).into_uuid() == uuid_


def test_timestamp_encode_decode() -> None:
    check_encode_decode_invariant(timestamps())


@given(st.datetimes())
def test_timestamp_from_into(datetime_: datetime.datetime) -> None:
    assert Timestamp.from_datetime(datetime_=datetime_).into_datetime() == datetime_


def test_epoch_timestamp_is_since_variant() -> None:
    """Ensure that when a datetime is exactly
    1970-01-01T00:00:00.000000000Z it is converted into a
    "since_epoch" protobuf Timestamp. We might state this
    circumstance in words "it has been 0ms since epoch".

    """
    epoch = datetime.datetime.utcfromtimestamp(0)
    timestamp = Timestamp.from_datetime(epoch)
    proto_timestamp = timestamp.into_proto()
    assert proto_timestamp.WhichOneof("duration") is not None
    assert proto_timestamp.WhichOneof("duration") == "since_epoch"


def test_duration_encode_decode() -> None:
    check_encode_decode_invariant(durations())


@given(st.timedeltas(min_value=datetime.timedelta(days=0)))
def test_duration_from_into(timedelta: datetime.timedelta) -> None:
    assert Duration.from_timedelta(timedelta=timedelta).into_timedelta() == timedelta
