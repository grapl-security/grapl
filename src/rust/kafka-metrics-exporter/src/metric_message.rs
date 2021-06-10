include!(concat!(env!("OUT_DIR"), "/metric_message.rs"));

impl From<Counter> for MetricMessage {
    fn from(counter: Counter) -> Self {
        MetricMessage {
            metric: Some(
                metric_message::Metric::Counter(counter)
            )
        }
    }
}


impl From<Gauge> for MetricMessage {
    fn from(gauge: Gauge) -> Self {
        MetricMessage {
            metric: Some(
                metric_message::Metric::Gauge(gauge)
            )
        }
    }
}


impl From<Histogram> for MetricMessage {
    fn from(histogram: Histogram) -> Self {
        MetricMessage {
            metric: Some(
                metric_message::Metric::Histogram(histogram)
            )
        }
    }
}
