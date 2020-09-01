use crate::metric_error::MetricError;
use crate::statsd_formatter::{statsd_format, MetricType, TagPair};

#[derive(Debug, Clone)]
pub struct MetricClient {
    /**
    So, this is a pretty odd struct. All it actually does is print things that look like
    MONITORING|<some_statsd_stuff_here>
    to stdout; then, later, a lambda reads in these messages and writes them to Cloudwatch.
    (originally recommended in an article by Yan Cui)
    */
    buffer: String,
}

/**
some followup TODOs:
    - add tags to the public functions (not needed right now)
*/
#[allow(dead_code)]
impl MetricClient {
    pub fn new() -> MetricClient {
        let buf: String = String::with_capacity(256);
        MetricClient { buffer: buf }
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

    pub fn counter(
        &mut self,
        metric_name: &str,
        value: f64,
        sample_rate: impl Into<Option<f64>>,
    ) -> Result<(), MetricError> {
        self.write_metric(metric_name, value, MetricType::Counter, sample_rate, &[])
    }

    /**
    A gauge is an instantaneous measurement of a value, like the gas gauge in a car.
    */
    pub fn gauge(&mut self, metric_name: &str, value: f64) -> Result<(), MetricError> {
        self.write_metric(metric_name, value, MetricType::Gauge, None, &[])
    }

    /**
    A histogram is a measure of the distribution of timer values over time, calculated at the
    server. As the data exported for timers and histograms is the same,
    this is currently an alias for a timer.

    example: the time to complete rendering of a web page for a user.
    */
    pub fn histogram(&mut self, metric_name: &str, value: f64) -> Result<(), MetricError> {
        self.write_metric(metric_name, value, MetricType::Histogram, None, &[])
    }
}

#[cfg(test)]
mod tests {
    use crate::metric_client::MetricClient;
    use crate::metric_error::MetricError;

    #[test]
    fn test_public_functions_smoke_test() -> Result<(), MetricError> {
        let mut mc = MetricClient::new();
        mc.histogram("metric_name", 123.45f64)?;
        mc.counter("metric_name", 123.45f64, None)?;
        mc.counter("metric_name", 123.45f64, 0.75)?;
        mc.gauge("metric_name", 123.45f64)?;
        Ok(())
    }
}
