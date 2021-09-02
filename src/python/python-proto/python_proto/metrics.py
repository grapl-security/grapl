from __future__ import annotations

import dataclasses
import enum
from typing import Sequence, Union, cast

from graplinc.grapl.metrics.v1beta1.types_pb2 import Counter as _Counter
from graplinc.grapl.metrics.v1beta1.types_pb2 import Gauge as _Gauge
from graplinc.grapl.metrics.v1beta1.types_pb2 import Histogram as _Histogram
from graplinc.grapl.metrics.v1beta1.types_pb2 import Label as _Label
from graplinc.grapl.metrics.v1beta1.types_pb2 import MetricWrapper as _MetricWrapper
from python_proto import SerDe


@dataclasses.dataclass(frozen=True)
class Label(SerDe):
    key: str
    value: str

    @staticmethod
    def deserialize(bytes_: bytes) -> Label:
        proto_label = _Label()
        proto_label.ParseFromString(bytes_)
        return Label.from_proto(proto_label=proto_label)

    def serialize(self) -> bytes:
        return cast(bytes, self.into_proto().SerializeToString())

    @staticmethod
    def from_proto(proto_label: _Label) -> Label:
        return Label(key=proto_label.key, value=proto_label.value)

    def into_proto(self) -> _Label:
        proto_label = _Label()
        proto_label.key = self.key
        proto_label.value = self.value
        return proto_label


@dataclasses.dataclass(frozen=True)
class Counter(SerDe):
    name: str
    increment: int
    labels: Sequence[Label]

    @staticmethod
    def deserialize(bytes_: bytes) -> Counter:
        proto_counter = _Counter()
        proto_counter.ParseFromString(bytes_)
        return Counter.from_proto(proto_counter=proto_counter)

    def serialize(self) -> bytes:
        return cast(bytes, self.into_proto().SerializeToString())

    @staticmethod
    def from_proto(proto_counter: _Counter) -> Counter:
        return Counter(
            name=proto_counter.name,
            increment=proto_counter.increment,
            labels=proto_counter.labels,
        )

    def into_proto(self) -> _Counter:
        proto_counter = _Counter()
        proto_counter.name = self.name
        proto_counter.increment = self.increment
        proto_counter.labels = self.labels
        return proto_counter


class GaugeType(enum.Enum):
    GAUGE_TYPE_UNSPECIFIED = "GAUGE_TYPE_UNSPECIFIED"
    GAUGE_TYPE_ABSOLUTE = "GAUGE_TYPE_ABSOLUTE"
    GAUGE_TYPE_INCREMENT = "GAUGE_TYPE_INCREMENT"
    GAUGE_TYPE_DECREMENT = "GAUGE_TYPE_DECREMENT"


@dataclasses.dataclass(frozen=True)
class Gauge(SerDe):
    gauge_type: GaugeType
    name: str
    value: float
    labels: Sequence[Label]

    @staticmethod
    def deserialize(bytes_: bytes) -> Gauge:
        proto_gauge = _Gauge()
        proto_gauge.ParseFromString(bytes_)
        return Gauge.from_proto(proto_gauge=proto_gauge)

    def serialize(self) -> bytes:
        return cast(bytes, self.into_proto().SerializeToString())

    @staticmethod
    def from_proto(proto_gauge: _Gauge) -> Gauge:
        return Gauge(
            gauge_type=GaugeType(proto_gauge.gauge_type),
            name=proto_gauge.name,
            value=proto_gauge.value,
            labels=proto_gauge.labels,
        )

    def into_proto(self) -> _Gauge:
        proto_gauge = _Gauge()
        proto_gauge.gauge_type = self.gauge_type.value
        proto_gauge.name = self.name
        proto_gauge.value = self.value
        proto_gauge.labels = self.labels
        return proto_gauge


@dataclasses.dataclass(frozen=True)
class Histogram(SerDe):
    name: str
    value: float
    labels: Sequence[Label]

    @staticmethod
    def deserialize(bytes_: bytes) -> Histogram:
        proto_histogram = _Histogram()
        proto_histogram.ParseFromString(bytes_)
        return Histogram.from_proto(proto_histogram=proto_histogram)

    def serialize(self) -> bytes:
        return cast(bytes, self.into_proto().SerializeToString())

    @staticmethod
    def from_proto(proto_histogram: _Histogram) -> Histogram:
        return Histogram(
            name=proto_histogram.name,
            value=proto_histogram.value,
            labels=proto_histogram.labels,
        )

    def into_proto(self) -> _Histogram:
        proto_histogram = _Histogram()
        proto_histogram.name = self.name
        proto_histogram.value = self.value
        proto_histogram.labels = self.labels
        return proto_histogram


@dataclasses.dataclass(frozen=True)
class MetricWrapper(SerDe):
    metric: Union[Counter, Gauge, Histogram]

    @staticmethod
    def deserialize(bytes_: bytes) -> MetricWrapper:
        proto_metric_wrapper = _MetricWrapper()
        proto_metric_wrapper.ParseFromString(bytes_)
        return MetricWrapper.from_proto(proto_metric_wrapper=proto_metric_wrapper)

    def serialize(self) -> bytes:
        return cast(bytes, self.into_proto().SerializeToString())

    @staticmethod
    def from_proto(proto_metric_wrapper: _MetricWrapper) -> MetricWrapper:
        metric_proto = proto_metric_wrapper.metric
        if isinstance(metric_proto, _Counter):
            metric = Counter.from_proto(proto_counter=metric_proto.metric)
        elif isinstance(metric_proto, _Gauge):
            metric = Gauge.from_proto(proto_gauge=metric_proto.metric)
        elif isinstance(metric_proto, _Histogram):
            metric = Histogram.from_proto(proto_histogram=metric_proto.metric)
        else:
            raise Exception(f"Encountered unknown type {metric_proto.metric}")
        return MetricWrapper(metric=metric)

    def into_proto(self) -> _MetricWrapper:
        proto_metric_wrapper = _MetricWrapper()
        proto_metric_wrapper.metric = self.metric.into_proto()
        return proto_metric_wrapper
