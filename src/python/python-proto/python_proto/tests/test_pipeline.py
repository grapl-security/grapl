import pytest

pytest.register_assert_rewrite("python_proto.tests.helpers")

from python_proto.tests.helpers import check_encode_decode_invariant
from python_proto.tests.strategies import envelopes, metadatas, raw_logs


def test_metadata_encode_decode() -> None:
    check_encode_decode_invariant(metadatas())


def test_envelope_encode_decode() -> None:
    check_encode_decode_invariant(envelopes())


def test_raw_log_encode_decode() -> None:
    check_encode_decode_invariant(raw_logs())
