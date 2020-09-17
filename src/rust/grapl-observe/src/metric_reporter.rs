use crate::metric_error::MetricError;
use crate::statsd_formatter;
use crate::statsd_formatter::{statsd_format, MetricType};
use chrono::{DateTime, SecondsFormat, Utc};
use std::fmt::Write;

pub mod common_strs {
    pub const STATUS: &'static str = "status";
    pub const SUCCESS: &'static str = "success";
    pub const FAIL: &'static str = "fail";
}

#[derive(Debug, Clone)]
pub struct MetricReporter {
    /**
    So, this is a pretty odd struct. All it actually does is print things that look like
    MONITORING|<some_statsd_stuff_here>
    to stdout; then, later, a lambda reads in these messages and writes them to Cloudwatch.
    (originally recommended in an article by Yan Cui)
    */
    // TODO: I think I gotta mutex or arc this; two threads grab it at once and sometimes print the wrong thing!
    buffer: String,
}

/**
some followup TODOs:
    - add tags to the public functions (not needed right now)
*/
#[allow(dead_code)]
impl MetricReporter {
    pub fn new() -> MetricReporter {
        let buf: String = String::new();
        MetricReporter { buffer: buf }
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
        // TODO: dependency-inject utcnow
        println!(
            "MONITORING|{}|{}",
            self.format_time_for_cloudwatch(Utc::now()),
            self.buffer
        );
        Ok(())
    }

    fn format_time_for_cloudwatch(&self, dt: DateTime<Utc>) -> String {
        // cloudwatch wants ISO8601, but without nanos.
        dt.to_rfc3339_opts(SecondsFormat::Millis, true)
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
    pub fn gauge_notags(&mut self, metric_name: &str, value: f64) -> Result<(), MetricError> {
        self.write_metric(metric_name, value, MetricType::Gauge, None, &[])
    }

    pub fn gauge(
        &mut self,
        metric_name: &str,
        value: f64,
        tags: &[TagPair],
    ) -> Result<(), MetricError> {
        self.write_metric(metric_name, value, MetricType::Gauge, None, tags)
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

pub struct TagPair<'a>(pub &'a str, pub &'a str);

impl TagPair<'_> {
    pub fn write_to_buf(&self, buf: &mut String) -> Result<(), MetricError> {
        let TagPair(tag_key, tag_value) = self;
        statsd_formatter::reject_invalid_chars(tag_key)?;
        statsd_formatter::reject_invalid_chars(tag_value)?;
        Ok(write!(buf, "{}={}", tag_key, tag_value)?)
    }
}

#[cfg(test)]
mod tests {
    use crate::metric_error::MetricError;
    use crate::metric_reporter::MetricReporter;
    use chrono::{DateTime, Utc};

    #[test]
    fn test_public_functions_smoke_test() -> Result<(), MetricError> {
        let mut reporter = MetricReporter::new();
        reporter.histogram("metric_name", 123.45f64)?;
        reporter.counter("metric_name", 123.45f64, None)?;
        reporter.counter("metric_name", 123.45f64, 0.75)?;
        reporter.gauge("metric_name", 123.45f64, &[])?;
        Ok(())
    }

    #[test]
    fn test_truncate_nanos() {
        let reporter = MetricReporter::new();
        let sample_with_nanos = "2020-09-16T18:53:16.985579647+00:00";
        let dt = DateTime::parse_from_rfc3339(sample_with_nanos)
            .expect("")
            .with_timezone(&Utc);
        let formatted = reporter.format_time_for_cloudwatch(dt);
        assert_eq!(formatted, "2020-09-16T18:53:16.985Z");
    }
}
