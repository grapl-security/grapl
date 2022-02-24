from __future__ import annotations

import dataclasses
import enum
from typing import Sequence, Type, Union, cast

from graplinc.grapl.metrics.v1.metric_types_pb2 import Counter as _Counter
from graplinc.grapl.metrics.v1.metric_types_pb2 import Gauge as _Gauge
from graplinc.grapl.metrics.v1.metric_types_pb2 import Histogram as _Histogram
from graplinc.grapl.metrics.v1.metric_types_pb2 import Label as _Label
from graplinc.grapl.metrics.v1.metric_types_pb2 import MetricWrapper as _MetricWrapper
from python_proto import SerDe


@dataclasses.dataclass(frozen=True)
class Label(SerDe[_Label]):
    key: str
    value: str
    proto_cls: Type[_Label] = _Label

    @staticmethod
    def deserialize(bytes_: bytes) -> Label:
        proto_label = _Label()
        proto_label.ParseFromString(bytes_)
        return Label.from_proto(proto_label=proto_label)

    @staticmethod
    def from_proto(proto_label: _Label) -> Label:
        return Label(key=proto_label.key, value=proto_label.value)

    def into_proto(self) -> _Label:
        proto_label = _Label()
        proto_label.key = self.key
        proto_label.value = self.value
        return proto_label


@dataclasses.dataclass(frozen=True)
class Counter(SerDe[_Counter]):
    name: str
    increment: int
    labels: Sequence[Label]
    proto_cls: Type[_Counter] = _Counter

    @staticmethod
    def deserialize(bytes_: bytes) -> Counter:
        proto_counter = _Counter()
        proto_counter.ParseFromString(bytes_)
        return Counter.from_proto(proto_counter=proto_counter)

    @staticmethod
    def from_proto(proto_counter: _Counter) -> Counter:
        return Counter(
            name=proto_counter.name,
            increment=proto_counter.increment,
            labels=[Label.from_proto(l) for l in proto_counter.labels],
        )

    def into_proto(self) -> _Counter:
        proto_counter = _Counter()
        proto_counter.name = self.name
        proto_counter.increment = self.increment
        for label in self.labels:
            proto_counter.labels.append(label.into_proto())
        return proto_counter


class GaugeType(enum.Enum):
    GAUGE_TYPE_UNSPECIFIED = "GAUGE_TYPE_UNSPECIFIED"
    GAUGE_TYPE_ABSOLUTE = "GAUGE_TYPE_ABSOLUTE"
    GAUGE_TYPE_INCREMENT = "GAUGE_TYPE_INCREMENT"
    GAUGE_TYPE_DECREMENT = "GAUGE_TYPE_DECREMENT"


@dataclasses.dataclass(frozen=True)
class Gauge(SerDe[_Gauge]):
    gauge_type: GaugeType
    name: str
    value: float
    labels: Sequence[Label]
    proto_cls: Type[_Gauge] = _Gauge

    @staticmethod
    def deserialize(bytes_: bytes) -> Gauge:
        proto_gauge = _Gauge()
        proto_gauge.ParseFromString(bytes_)
        return Gauge.from_proto(proto_gauge=proto_gauge)

    @staticmethod
    def from_proto(proto_gauge: _Gauge) -> Gauge:
        return Gauge(
            gauge_type=GaugeType(_Gauge.GaugeType.Name(proto_gauge.gauge_type)),
            name=proto_gauge.name,
            value=proto_gauge.value,
            labels=[Label.from_proto(l) for l in proto_gauge.labels],
        )

    def into_proto(self) -> _Gauge:
        proto_gauge = _Gauge()
        proto_gauge.gauge_type = _Gauge.GaugeType.Value(self.gauge_type.value)
        proto_gauge.name = self.name
        proto_gauge.value = self.value
        for label in self.labels:
            proto_gauge.labels.append(label.into_proto())
        return proto_gauge


@dataclasses.dataclass(frozen=True)
class Histogram(SerDe):
    name: str
    value: float
    labels: Sequence[Label]
    proto_cls: Type[_Histogram] = _Histogram

    @staticmethod
    def deserialize(bytes_: bytes) -> Histogram:
        proto_histogram = _Histogram()
        proto_histogram.ParseFromString(bytes_)
        return Histogram.from_proto(proto_histogram=proto_histogram)

    @staticmethod
    def from_proto(proto_histogram: _Histogram) -> Histogram:
        return Histogram(
            name=proto_histogram.name,
            value=proto_histogram.value,
            labels=[Label.from_proto(l) for l in proto_histogram.labels],
        )

    def into_proto(self) -> _Histogram:
        proto_histogram = _Histogram()
        proto_histogram.name = self.name
        proto_histogram.value = self.value
        for label in self.labels:
            proto_histogram.labels.append(label.into_proto())
        return proto_histogram


@dataclasses.dataclass(frozen=True)
class MetricWrapper(SerDe):
    metric: Union[Counter, Gauge, Histogram]
    proto_cls: Type[_MetricWrapper] = _MetricWrapper

    @staticmethod
    def deserialize(bytes_: bytes) -> MetricWrapper:
        proto_metric_wrapper = _MetricWrapper()
        proto_metric_wrapper.ParseFromString(bytes_)
        return MetricWrapper.from_proto(proto_metric_wrapper=proto_metric_wrapper)

    @staticmethod
    def from_proto(proto_metric_wrapper: _MetricWrapper) -> MetricWrapper:
        if proto_metric_wrapper.HasField("counter"):
            metric = cast(
                Union[Counter, Gauge, Histogram],
                Counter.from_proto(proto_counter=proto_metric_wrapper.counter),
            )
        elif proto_metric_wrapper.HasField("gauge"):
            metric = cast(
                Union[Counter, Gauge, Histogram],
                Gauge.from_proto(proto_gauge=proto_metric_wrapper.gauge),
            )
        elif proto_metric_wrapper.HasField("histogram"):
            metric = cast(
                Union[Counter, Gauge, Histogram],
                Histogram.from_proto(proto_histogram=proto_metric_wrapper.histogram),
            )
        else:
            raise Exception("Encountered unknown type")
        return MetricWrapper(metric=metric)

    def into_proto(self) -> _MetricWrapper:
        proto_metric_wrapper = _MetricWrapper()
        if type(self.metric) is Counter:
            proto_metric_wrapper.counter.CopyFrom(
                cast(_Counter, self.metric.into_proto())
            )
        elif type(self.metric) is Gauge:
            proto_metric_wrapper.gauge.CopyFrom(cast(_Gauge, self.metric.into_proto()))
        elif type(self.metric) is Histogram:
            proto_metric_wrapper.histogram.CopyFrom(
                cast(_Histogram, self.metric.into_proto())
            )
        return proto_metric_wrapper
