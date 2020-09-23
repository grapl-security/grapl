from contextlib import ContextManager
from grapl_common.metrics.metric_reporter import MetricReporter, TagPair
from typing_extensions import Literal


class EngagementCreatorMetrics:
    def __init__(self, service_name: str) -> None:
        self.metric_reporter = MetricReporter.create(service_name)

    def event_processed(self, status: Literal["success", "failure"]) -> None:
        self.metric_reporter.counter(
            metric_name="event_processed", value=1, tags=[TagPair("status", status)]
        )

    def time_to_process_event(self) -> ContextManager:
        return self.metric_reporter.histogram_ctx(metric_name="time_to_process_event")
