use crate::metric_error::MetricError;
use crate::statsd_formatter;
use crate::statsd_formatter::{statsd_format, MetricType};
use crate::writer_wrapper::WriterWrapper;
use chrono::{DateTime, SecondsFormat, Utc};
use std::fmt::Write;
use std::io::{stdout, Stdout};

pub mod common_strs {
    pub const STATUS: &'static str = "status";
    pub const SUCCESS: &'static str = "success";
    pub const FAIL: &'static str = "fail";
}

type NowGetter = fn() -> DateTime<Utc>;

pub struct MetricReporter<W: std::io::Write> {
    /**
    So, this is a pretty odd struct. All it actually does is print things that look like
    MONITORING|service_name|timestamp|<some_statsd_stuff_here>
    to stdout; then, later, a lambda reads in these messages and writes them to Cloudwatch.
    (originally recommended in an article by Yan Cui)
    */
    buffer: String,
    out: WriterWrapper<W>,
    utc_now: NowGetter,
    service_name: String,
}

/**
some followup TODOs:
    - add tags to the public functions (not needed right now)
*/
#[allow(dead_code)]
impl<W> MetricReporter<W>
where
    W: std::io::Write,
{
    pub fn new(service_name: &str) -> MetricReporter<Stdout> {
        MetricReporter {
            service_name: service_name.to_string(),
            buffer: String::new(),
            out: WriterWrapper::new(stdout()),
            utc_now: Utc::now,
        }
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
        let time = self.format_time_for_cloudwatch((self.utc_now)());
        write!(
            self.out.as_mut(),
            "MONITORING|{}|{}|{}\n",
            self.service_name,
            time,
            self.buffer
        )?;
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
    pub fn histogram(
        &mut self,
        metric_name: &str,
        value: f64,
        tags: &[TagPair],
    ) -> Result<(), MetricError> {
        self.write_metric(metric_name, value, MetricType::Histogram, None, tags)
    }
}

impl Clone for MetricReporter<Vec<u8>> {
    fn clone(&self) -> Self {
        Self {
            buffer: self.buffer.clone(),
            out: self.out.clone(),
            utc_now: self.utc_now.clone(),
            service_name: self.service_name.clone(),
        }
    }
}

impl Clone for MetricReporter<Stdout> {
    fn clone(&self) -> Self {
        Self {
            buffer: self.buffer.clone(),
            out: self.out.clone(),
            utc_now: self.utc_now.clone(),
            service_name: self.service_name.clone(),
        }
    }
}

pub struct TagPair<'a>(pub &'a str, pub &'a str);

impl TagPair<'_> {
    pub fn write_to_buf(&self, buf: &mut String) -> Result<(), MetricError> {
        let TagPair(tag_key, tag_value) = self;
        statsd_formatter::reject_invalid_chars(tag_key)?;
        statsd_formatter::reject_invalid_chars(tag_value)?;
        Ok(write!(buf, "{}:{}", tag_key, tag_value)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_utc() -> DateTime<Utc> {
        let sample_with_nanos = "2020-01-01T01:23:45Z";
        DateTime::parse_from_rfc3339(sample_with_nanos)
            .expect("")
            .with_timezone(&Utc)
    }

    const SERVICE_NAME: &'static str = "test_service";

    #[test]
    fn test_public_functions_smoke_test() -> Result<(), Box<dyn std::error::Error>> {
        let vec_writer: WriterWrapper<Vec<u8>> = WriterWrapper::new(vec![]);
        let mut reporter = MetricReporter {
            buffer: String::new(),
            out: vec_writer,
            utc_now: test_utc,
            service_name: SERVICE_NAME.to_string(),
        };
        reporter.histogram("metric_name", 123.45f64, &[])?;
        reporter.counter("metric_name", 123.45f64, None)?;
        reporter.counter("metric_name", 123.45f64, 0.75)?;
        reporter.gauge("metric_name", 123.45f64, &[TagPair("key", "value")])?;
        let vec = reporter.out.release();

        let written = String::from_utf8(vec)?;
        let expected: Vec<&str> = vec![
            "MONITORING|test_service|2020-01-01T01:23:45.000Z|metric_name:123.45|h",
            "MONITORING|test_service|2020-01-01T01:23:45.000Z|metric_name:123.45|c",
            "MONITORING|test_service|2020-01-01T01:23:45.000Z|metric_name:123.45|c|@0.75",
            "MONITORING|test_service|2020-01-01T01:23:45.000Z|metric_name:123.45|g|#key:value",
        ];
        let actual: Vec<&str> = written.split("\n").collect();
        for (expected, actual) in expected.iter().zip(actual.iter()) {
            assert_eq!(expected, actual)
        }
        Ok(())
    }

    #[test]
    fn test_truncate_nanos() {
        let reporter = MetricReporter::<Stdout>::new(SERVICE_NAME);
        let sample_with_nanos = "2020-09-16T18:53:16.985579647+00:00";
        let dt = DateTime::parse_from_rfc3339(sample_with_nanos)
            .expect("")
            .with_timezone(&Utc);
        let formatted = reporter.format_time_for_cloudwatch(dt);
        assert_eq!(formatted, "2020-09-16T18:53:16.985Z");
    }
}
