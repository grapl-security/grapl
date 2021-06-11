include!(concat!(env!("OUT_DIR"), "/graplinc.grapl.metrics.v1.rs"));

impl From<Counter> for MetricWrapper {
    fn from(counter: Counter) -> Self {
        MetricWrapper {
            metric: Some(metric_wrapper::Metric::Counter(counter)),
        }
    }
}

impl From<Gauge> for MetricWrapper {
    fn from(gauge: Gauge) -> Self {
        MetricWrapper {
            metric: Some(metric_wrapper::Metric::Gauge(gauge)),
        }
    }
}

impl From<Histogram> for MetricWrapper {
    fn from(histogram: Histogram) -> Self {
        MetricWrapper {
            metric: Some(metric_wrapper::Metric::Histogram(histogram)),
        }
    }
}
