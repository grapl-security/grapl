import uuid

import pytest

pytest.register_assert_rewrite("python_proto.tests.helpers")

from hypothesis import given
from hypothesis import strategies as st
from python_proto.pipeline import Uuid
from python_proto.tests.helpers import check_encode_decode_invariant
from python_proto.tests.strategies import envelopes, metadatas, uuids


def test_uuid_encode_decode() -> None:
    check_encode_decode_invariant(uuids())


@given(st.uuids())
def test_uuid_from_into(uuid_: uuid.UUID) -> None:
    assert Uuid.from_uuid(uuid_).into_uuid() == uuid_


def test_metadata_encode_decode() -> None:
    check_encode_decode_invariant(metadatas())


def test_envelope_encode_decode() -> None:
    check_encode_decode_invariant(envelopes())
