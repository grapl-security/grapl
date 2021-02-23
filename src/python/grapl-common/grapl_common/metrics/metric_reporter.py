from contextlib import contextmanager
from datetime import datetime, timezone
from enum import Enum
from sys import stdout
from typing import Callable, Iterator, Optional, Sequence, TextIO, Union

from grapl_common.metrics.statsd_formatter import (
    DEFAULT_SAMPLE_RATE,
    MetricType,
    TagPair,
    statsd_format,
)
from grapl_common.time_utils import MillisDuration, as_millis_duration
from typing_extensions import Protocol


class Writeable(Protocol):
    """
    Protocol for `stdout`.
    Like a TextIO but with less to mock out.
    """

    def write(self, some_str: str) -> int:
        pass


_RESERVED_UNIT_TAG = "_unit"


class HistogramUnit(str, Enum):
    MILLIS = "millis"
    MICROS = "micros"
    SECONDS = "seconds"


class MetricReporter:
    """
    Print metrics to stdout that are picked up by Metric Forwarder later.
    Prefer MetricReporter.create(my_service_name) to get an instance.
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

    def counter(
        self,
        metric_name: str,
        value: Union[int, float],
        sample_rate: float = DEFAULT_SAMPLE_RATE,
        tags: Sequence[TagPair] = (),
    ) -> None:
        """
        Sort of like a gauge, but with deltas.
        Metrics sent will increment or decrement the value of the gauge rather than giving its current value.
        """
        self.write_metric(
            metric_name=metric_name,
            value=value,
            sample_rate=sample_rate,
            typ="c",
            tags=tags,
        )

    def gauge(
        self,
        metric_name: str,
        value: Union[int, float],
        tags: Sequence[TagPair] = (),
    ) -> None:
        """
        An instantaneous measurement of a value, like the gas gauge in a car.
        """
        self.write_metric(
            metric_name=metric_name,
            value=value,
            typ="g",
            tags=tags,
        )

    def histogram(
        self,
        metric_name: str,
        value: MillisDuration,
        tags: Sequence[TagPair] = (),
        unit: Optional[HistogramUnit] = None,
    ) -> None:
        """
        A histogram is a measure of the distribution of timer values over time, calculated at the
        server. As the data exported for timers and histograms is the same,
        this is currently an alias for a timer.

        example: the time to complete rendering of a web page for a user.
        """
        if unit:
            tags = [TagPair(_RESERVED_UNIT_TAG, unit.value)] + [t for t in tags]
        self.write_metric(
            metric_name=metric_name,
            value=value,
            typ="h",
            tags=tags,
        )

    @contextmanager
    def histogram_ctx(
        self,
        metric_name: str,
        tags: Sequence[TagPair] = (),
    ) -> Iterator[None]:
        """
        A histogram is a measure of the distribution of timer values over time, calculated at the
        server. As the data exported for timers and histograms is the same,
        this is currently an alias for a timer.

        example: the time to complete rendering of a web page for a user.
        """
        start = self.utc_now()
        try:
            yield
        finally:
            end = self.utc_now()
            value = as_millis_duration(end - start)
            self.write_metric(
                metric_name=metric_name,
                value=value,
                typ="h",
                tags=tags,
            )

    _TIME_SPEC = "milliseconds"

    @staticmethod
    def _format_time_for_cloudwatch(dt: datetime) -> str:
        return dt.isoformat(timespec=MetricReporter._TIME_SPEC)

    @staticmethod
    def _utcnow() -> datetime:
        return datetime.now(timezone.utc)
