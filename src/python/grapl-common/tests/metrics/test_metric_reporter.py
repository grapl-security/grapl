from datetime import datetime
from typing import List
from grapl_common.metrics.metric_reporter import MetricReporter, TagPair
from grapl_common.time_utils import MillisDuration


class TestMetricReporter:
    def test__smoke_test(self) -> None:
        f = Fixture()
        tags = (TagPair("k1", "v1"), TagPair("k2", "v2"))
        f.metric_reporter.gauge("some_metric", 1.0, tags=tags)
        f.metric_reporter.counter("some_metric", 2.0, tags=tags)
        f.metric_reporter.counter("some_metric", 3.0, sample_rate=0.5, tags=tags)
        f.metric_reporter.histogram("some_metric", MillisDuration(4), tags=tags)
        assert f.out.writes == [
            "MONITORING|py_test_service|2020-09-20T01:02:03.004|some_metric:1.0|g|#k1:v1,k2:v2\n",
            "MONITORING|py_test_service|2020-09-20T01:02:03.004|some_metric:2.0|c|#k1:v1,k2:v2\n",
            "MONITORING|py_test_service|2020-09-20T01:02:03.004|some_metric:3.0|c|@0.5|#k1:v1,k2:v2\n",
            "MONITORING|py_test_service|2020-09-20T01:02:03.004|some_metric:4|h|#k1:v1,k2:v2\n",
        ]

    def test__histogram_ctx(self) -> None:
        f = Fixture()

        dts = [
            # start
            datetime(
                year=2020, month=9, day=20, hour=1, minute=2, second=3, microsecond=1000
            ),
            # end, which is +3 milliseconds
            datetime(
                year=2020, month=9, day=20, hour=1, minute=2, second=3, microsecond=4000
            ),
            # this last one doesn't matter for measuring delta, just the timestamp
            datetime(
                year=2020, month=9, day=20, hour=1, minute=2, second=3, microsecond=8000
            ),
        ]

        def return_dts() -> datetime:
            return dts.pop(0)

        f.metric_reporter.utc_now = return_dts

        # histogram_ctx should get the two values
        with f.metric_reporter.histogram_ctx("some_metric"):
            pass

        assert f.out.writes == [
            "MONITORING|py_test_service|2020-09-20T01:02:03.008|some_metric:3|h\n",
        ]


class Fixture:
    def __init__(self) -> None:
        self.service_name = "py_test_service"
        self.out = MockWriteable()
        utc_now = lambda: datetime(
            year=2020, month=9, day=20, hour=1, minute=2, second=3, microsecond=4000
        )
        self.metric_reporter = MetricReporter(
            service_name=self.service_name, out=self.out, utc_now=utc_now
        )


class MockWriteable:
    def __init__(self) -> None:
        self.writes: List[str] = []

    def write(self, some_str: str) -> int:
        self.writes.append(some_str)
        return 0
