from typing_extensions import Protocol
from typing import Optional, Callable, Sequence, Union, TextIO
from datetime import datetime, timezone
from sys import stdout
from grapl_common.metrics.statsd_formatter import (
    MetricType,
    DEFAULT_SAMPLE_RATE,
    statsd_format,
    TagPair,
)


class Writeable(Protocol):
    def write(self, some_str: str) -> int:
        pass


class MetricReporter:
    """
    Use MetricReporter.get(my_service_name).
    """

    def __init__(
        self,
        service_name: str,
        utc_now: Callable[[], datetime],
        out: Writeable,
    ):
        self.service_name = service_name
        self.utc_now = utc_now
        self.out = out

    @staticmethod
    def create(service_name: str) -> "MetricReporter":
        return MetricReporter(service_name, utc_now=MetricReporter._utcnow, out=stdout)

    def write_metric(
        self,
        metric_name: str,
        value: Union[int, float],
        typ: MetricType,
        sample_rate: float = DEFAULT_SAMPLE_RATE,
        tags: Sequence[TagPair] = (),
    ) -> None:
        now = self.utc_now()
        now_ts = self._format_time_for_cloudwatch(now)
        statsd = statsd_format(
            metric_name,
            value,
            typ,
            sample_rate,
            tags,
        )
        self.out.write(f"MONITORING|{self.service_name}|{now_ts}|{statsd}\n")

    _TIME_SPEC = "milliseconds"

    @staticmethod
    def _format_time_for_cloudwatch(dt: datetime) -> str:
        return dt.isoformat(timespec=MetricReporter._TIME_SPEC)

    @staticmethod
    def _utcnow() -> datetime:
        return datetime.now(timezone.utc)
