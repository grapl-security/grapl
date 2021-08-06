use std::fmt::Write;

use lazy_static::lazy_static;
use regex::Regex;

use crate::{
    metric_error::{
        MetricError,
        MetricError::{
            InvalidCharacter,
            InvalidSampleRate,
        },
    },
    metric_reporter::TagPair,
};

lazy_static! {
    static ref INVALID_CHARS: Regex = Regex::new("[|#,=:]").unwrap();
}

pub fn reject_invalid_chars(s: &str) -> Result<(), MetricError> {
    let matched = INVALID_CHARS.is_match(s);
    if matched {
        Err(InvalidCharacter())
    } else {
        Ok(())
    }
}

pub enum MetricType {
    Gauge,
    Counter,
    Histogram,
}

const GAUGE_STR: &'static str = "g";
const COUNTER_STR: &'static str = "c";
const HISTOGRAM_STR: &'static str = "h";

impl MetricType {
    fn statsd_type(&self) -> &'static str {
        match self {
            MetricType::Gauge => GAUGE_STR,
            MetricType::Counter => COUNTER_STR,
            MetricType::Histogram => HISTOGRAM_STR,
        }
    }
}

/**
Don't call statsd_format directly; instead, prefer the public functions of MetricClient.
To go from a formatted string to usable data again, use the 'statsd-parser' crate.
*/
#[allow(dead_code)]
pub fn statsd_format(
    buf: &mut String,
    metric_name: &str,
    value: f64,
    metric_type: MetricType,
    sample_rate: impl Into<Option<f64>>,
    tags: &[TagPair],
) -> Result<(), MetricError> {
    buf.clear();
    reject_invalid_chars(metric_name)?;

    write!(
        buf,
        "{metric_name}:{value}|{metric_type}",
        metric_name = metric_name,
        value = value,
        metric_type = metric_type.statsd_type()
    )?;

    match (metric_type, sample_rate.into()) {
        (MetricType::Counter, Some(rate)) => {
            // a rate of 1.0 we'll just ignore
            if rate >= 0.0 && rate < 1.0 {
                write!(buf, "|@{sample_rate}", sample_rate = rate)?;
            } else {
                return Err(InvalidSampleRate());
            }
        }
        _ => {}
    }

    let mut first_tag: bool = true;
    if !tags.is_empty() {
        // begin tag section
        write!(buf, "|#")?;
        for pair in tags {
            // separator
            if !first_tag {
                write!(buf, ",")?;
            } else {
                first_tag = false;
            }
            pair.write_to_buf(buf)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{
        metric_error::MetricError,
        statsd_formatter::{
            reject_invalid_chars,
            statsd_format,
            MetricType,
            TagPair,
        },
    };

    const INVALID_STRS: [&str; 5] = [
        "some|metric",
        "some#metric",
        "some,metric",
        "some:metric",
        "some=metric",
    ];

    const VALID_STR: &str = "some_str";
    const VALID_VALUE: f64 = 12345.6;

    fn make_tags() -> Vec<TagPair<'static>> {
        vec![
            TagPair("some_key", "some_value"),
            TagPair("some_key_2", "some_value_2"),
        ]
    }

    fn make_empty_tags() -> [TagPair<'static>; 0] {
        let empty_slice: [TagPair<'static>; 0] = [];
        return empty_slice;
    }

    #[test]
    fn test_reject_invalid_chars() -> Result<(), String> {
        for invalid_str in INVALID_STRS.iter() {
            let result = reject_invalid_chars(invalid_str);
            match result.expect_err("else panic") {
                MetricError::InvalidCharacter() => Ok(()),
                _ => Err(String::from("expected invalid character error")),
            }?
        }

        assert!(reject_invalid_chars(VALID_STR).is_ok());
        Ok(())
    }

    #[test]
    fn test_statsd_format_basic_counter() -> Result<(), MetricError> {
        let mut buf: String = String::with_capacity(256);
        statsd_format(
            &mut buf,
            VALID_STR,
            VALID_VALUE,
            MetricType::Counter,
            None,
            &make_empty_tags(),
        )?;
        assert_eq!(buf, "some_str:12345.6|c");
        Ok(())
    }

    #[test]
    fn test_statsd_format_specify_rate() -> Result<(), MetricError> {
        let mut buf: String = String::with_capacity(256);
        statsd_format(
            &mut buf,
            VALID_STR,
            VALID_VALUE,
            MetricType::Counter,
            0.5,
            &make_empty_tags(),
        )?;
        assert_eq!(buf, "some_str:12345.6|c|@0.5");
        Ok(())
    }

    #[test]
    fn test_statsd_format_specify_bad_rate() -> Result<(), String> {
        let mut buf: String = String::with_capacity(256);
        let result = statsd_format(
            &mut buf,
            VALID_STR,
            VALID_VALUE,
            MetricType::Counter,
            1.5,
            &make_empty_tags(),
        );
        match result.expect_err("") {
            MetricError::InvalidSampleRate() => Ok(()),
            _ => Err(String::from("unexpected err")),
        }
    }

    #[test]
    fn test_statsd_format_tags() -> Result<(), MetricError> {
        let mut buf: String = String::with_capacity(256);
        statsd_format(
            &mut buf,
            VALID_STR,
            VALID_VALUE,
            MetricType::Counter,
            None,
            &make_tags(),
        )?;
        assert_eq!(
            buf,
            "some_str:12345.6|c|#some_key:some_value,some_key_2:some_value_2"
        );
        Ok(())
    }
    #[test]
    fn test_statsd_format_bad_tags() -> Result<(), String> {
        let mut buf: String = String::with_capacity(256);
        let result = statsd_format(
            &mut buf,
            VALID_STR,
            VALID_VALUE,
            MetricType::Counter,
            None,
            &[TagPair("some|key", "val")],
        );
        match result.expect_err("") {
            MetricError::InvalidCharacter() => Ok(()),
            _ => Err(String::from("unexpected err")),
        }
    }
}
