import pytest

pytest.register_assert_rewrite("python_proto.tests.helpers")

from python_proto.tests.helpers import check_encode_decode_invariant
from python_proto.tests.strategies import (
    counters,
    gauges,
    histograms,
    labels,
    metric_wrappers,
)


def test_label_encode_decode() -> None:
    check_encode_decode_invariant(labels())


def test_counter_encode_decode() -> None:
    check_encode_decode_invariant(counters())


def test_gauge_encode_decode() -> None:
    check_encode_decode_invariant(gauges())


def test_histogram_encode_decode() -> None:
    check_encode_decode_invariant(histograms())


def test_metric_wrapper_encode_decode() -> None:
    check_encode_decode_invariant(metric_wrappers())
