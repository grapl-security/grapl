import unittest
from typing import Sequence

import pytest

from grapl_common.metrics.statsd_formatter import TagPair, statsd_format


class TestData:
    METRIC_NAME = "some_metric"
    VALUE = 2.0
    SAMPLE_RATE = 0.5
    TAGS = (
        TagPair("key", "value"),
        TagPair("key2", "value2"),
    )

    VALID_STR = "some_str"
    INVALID_STRS: Sequence[str] = (
        "some|metric",
        "some#metric",
        "some,metric",
        "some:metric",
        "some=metric",
    )


class TestStatsdFormatter(unittest.TestCase):
    def test_basic_counter(self):
        result = statsd_format(
            metric_name=TestData.METRIC_NAME,
            value=TestData.VALUE,
            typ="c",
        )
        assert result == "some_metric:2.0|c"

    def test_counter_with_sample_rate(self):
        result = statsd_format(
            metric_name=TestData.METRIC_NAME,
            value=TestData.VALUE,
            sample_rate=TestData.SAMPLE_RATE,
            typ="c",
        )
        assert result == "some_metric:2.0|c|@0.5"

    def test_non_counter_with_sample_rate_doesnt_include_it(self):
        result = statsd_format(
            metric_name=TestData.METRIC_NAME,
            value=TestData.VALUE,
            sample_rate=TestData.SAMPLE_RATE,
            typ="g",
        )
        assert result == "some_metric:2.0|g"

    def test_tags(self):
        result = statsd_format(
            metric_name=TestData.METRIC_NAME,
            value=TestData.VALUE,
            sample_rate=TestData.SAMPLE_RATE,
            typ="ms",
            tags=TestData.TAGS,
        )
        assert result == "some_metric:2.0|ms|#key=value,key2=value2"

    def test_invalid_metric_names(self):
        for invalid_metric_name in TestData.INVALID_STRS:
            with pytest.raises(ValueError):
                statsd_format(
                    metric_name=invalid_metric_name,
                    value=TestData.VALUE,
                    typ="c",
                )

    def test_invalid_tag_keys_and_values(self):
        for invalid_str in TestData.INVALID_STRS:
            # mutate key, then value
            with pytest.raises(ValueError):
                TagPair(invalid_str, TestData.VALID_STR)
            with pytest.raises(ValueError):
                TagPair(TestData.VALID_STR, invalid_str)
