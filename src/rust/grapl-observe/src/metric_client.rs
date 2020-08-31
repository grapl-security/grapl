use crate::metric_error::MetricError;
use crate::statsd_formatter::{statsd_format, MetricType, TagPair};

pub struct MetricClient {
    buffer: String,
}

/**
some followup TODOs:
    - add tags to the public functions (not needed right now)
    - is there any need for the `ms` metric type?
*/
impl MetricClient {
    pub fn new() -> MetricClient {
        let mut buf: String = String::with_capacity(256);
        return MetricClient { buffer: buf };
    }

    fn write_metric(
        &mut self,
        metric_name: &str,
        value: f64,
        metric_type: MetricType,
        sample_rate: impl Into<Option<f64>>,
        tags: &[TagPair],
    ) -> Result<(), MetricError> {
        statsd_format(
            &mut self.buffer,
            metric_name,
            value,
            metric_type,
            sample_rate,
            tags,
        )?;
        println!("MONITORING|{}", self.buffer);
        Ok(())
    }

    pub fn counter(&mut self, metric_name: &str, value: f64, sample_rate: impl Into<Option<f64>>) {
        self.write_metric(metric_name, value, MetricType::Counter, sample_rate, &[]);
    }

    pub fn gauge(&mut self, metric_name: &str, value: f64) {
        self.write_metric(metric_name, value, MetricType::Gauge, None, &[]);
    }

    pub fn histogram(&mut self, metric_name: &str, value: f64) {
        self.write_metric(metric_name, value, MetricType::Histogram, None, &[]);
    }
}

#[cfg(test)]
mod tests {
    use crate::metric_client::MetricClient;
    #[test]
    fn test_public_functions_smoke_test() {
        let mut mc = MetricClient::new();
        mc.histogram("metric_name", 123.45f64);
        mc.counter("metric_name", 123.45f64, None);
    }
}
