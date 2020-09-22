from datetime import datetime
from grapl_common.metrics.metric_reporter import MetricReporter, TagPair


class MetricReporterTests:
    def test__smoke_test(self):
        f = Fixture()
        tags = (TagPair("k1", "v1"), TagPair("k2", "v2"))
        f.metric_reporter.gauge("some_metric", 1.0, tags=tags)
        f.metric_reporter.counter("some_metric", 2.0, tags=tags)
        f.metric_reporter.counter("some_metric", 3.0, sample_rate=0.5, tags=tags)
        f.metric_reporter.counter("some_metric", 4.0, tags=tags)
        assert f.out.writes == [
            "MONITORING|py_test_service|20200920T01:02:03.4000Z|some_metric:1.0|g|#k1:v1,k2:v2"
            "MONITORING|py_test_service|20200920T01:02:03.4000Z|some_metric:2.0|c|#k1:v1,k2:v2"
            "MONITORING|py_test_service|20200920T01:02:03.4000Z|some_metric:3.0|c|@0.5|#k1:v1,k2:v2"
            "MONITORING|py_test_service|20200920T01:02:03.4000Z|some_metric:4.0|h|#k1:v1,k2:v2"
        ]


class Fixture:
    def __init__(self):
        self.service_name = "py_test_service"
        self.out = MockWriteable()
        self.utc_now = lambda: datetime(
            year=2020, month=9, day=20, hour=1, minute=2, second=3, microsecond=4000
        )
        self.metric_reporter = MetricReporter(
            service_name=self.service_name, out=self.out, utc_now=self.utc_now
        )


class MockWriteable:
    def __init__(self):
        self.writes: List[str] = []

    def write(self, some_str: str) -> int:
        self.writes.append(some_str)
        return 0
