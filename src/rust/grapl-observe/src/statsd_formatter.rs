use crate::metric_error::MetricError;
use crate::metric_error::MetricError::{MetricInvalidCharacterError, MetricInvalidSampleRateError};
use lazy_static::lazy_static;
use regex::Regex;
use std::fmt::Write;

lazy_static! {
    static ref INVALID_CHARS: Regex = Regex::new("[|#,=:]").unwrap();
}

pub struct TagPair<'a> {
    tag_key: &'a str,
    tag_value: &'a str,
}

impl TagPair<'_> {
    fn statsd_serialized(&self) -> String {
        return format!("{}={}", self.tag_key, self.tag_value);
    }
}

fn reject_invalid_chars(s: &str) -> Result<(), MetricError> {
    let matched = INVALID_CHARS.is_match(s);
    if matched {
        Err(MetricInvalidCharacterError())
    } else {
        Ok(())
    }
}

pub enum MetricType {
    Gauge,
    Counter,
    Millis,
    Histogram,
}

impl MetricType {
    fn statsd_type(&self) -> &'static str {
        let g: &'static str = "g";
        let c: &'static str = "c";
        let ms: &'static str = "ms";
        let h: &'static str = "h";

        match self {
            MetricType::Gauge => g,
            MetricType::Counter => c,
            MetricType::Millis => ms,
            MetricType::Histogram => h,
        }
    }
}

pub fn statsd_format(
    metric_name: &str,
    value: f64, // should i make a union type?
    metric_type: MetricType,
    sample_rate: Option<f64>,
    tags: &[TagPair],
) -> Result<String, MetricError> {
    // initial capacity chosen relatively arbitrarily
    let mut buf: String = String::with_capacity(256);

    reject_invalid_chars(metric_name)?;

    write!(
        buf,
        "{metric_name}:{value}|{metric_type}",
        metric_name = metric_name,
        value = value,
        metric_type = metric_type.statsd_type()
    )?;

    match (metric_type, sample_rate) {
        (MetricType::Counter, Some(rate)) => {
            // a rate of 1.0 we'll just ignore
            if rate >= 0.0 && rate < 1.0 {
                write!(buf, "|@{sample_rate}", sample_rate = rate)?;
            } else {
                return Err(MetricInvalidSampleRateError());
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
            write!(buf, "{}", pair.statsd_serialized())?
        }
    }
    return Ok(buf);
}

#[cfg(test)]
mod tests {
    use crate::metric_error::MetricError;
    use crate::statsd_formatter::{reject_invalid_chars, statsd_format, MetricType, TagPair};

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
            TagPair {
                tag_key: "some_key",
                tag_value: "some_value",
            },
            TagPair {
                tag_key: "some_key_2",
                tag_value: "some_value_2",
            },
        ]
    }

    fn make_empty_tags() -> [TagPair<'static>; 0] {
        let empty_slice: [TagPair<'static>; 0] = [];
        return empty_slice;
    }

    #[test]
    fn test_reject_invalid_chars() {
        INVALID_STRS.iter().for_each(|invalid_str| {
            let result = reject_invalid_chars(invalid_str);
            match result.expect_err("else panic") {
                MetricError::MetricInvalidCharacterError() => { /* what we want */ }
                _ => panic!(),
            }
        });

        assert!(reject_invalid_chars(VALID_STR).is_ok())
    }

    #[test]
    fn test_statsd_format_basic_counter() {
        let result = statsd_format(
            VALID_STR,
            VALID_VALUE,
            MetricType::Counter,
            None,
            &make_empty_tags(),
        )
        .unwrap();
        assert_eq!(result, "some_str:12345.6|c")
    }

    #[test]
    fn test_statsd_format_tags() {
        let result = statsd_format(
            VALID_STR,
            VALID_VALUE,
            MetricType::Counter,
            None,
            &make_tags(),
        )
        .unwrap();
        assert_eq!(
            result,
            "some_str:12345.6|c|#some_key=some_value,some_key_2=some_value_2"
        )
    }
}
