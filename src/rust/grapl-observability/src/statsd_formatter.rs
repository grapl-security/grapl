use lazy_static::lazy_static;
use regex::Regex;
use failure::Error;
use std::fmt;
use std::error::{Error as StdError};

#[derive(Debug)]
struct MetricError {
    // TODO: should this own a string or just have a reference to it?
}

impl StdError for MetricError {}

impl fmt::Display for MetricError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Bad stuff with Metrics")
    }
}

lazy_static! {
    static ref INVALID_CHARS: Regex = Regex::new("[|#,=:]").unwrap();
}
// const DEFAULT_SAMPLE_RATE: f64 = 1.0;



fn reject_invalid_chars(s: &str) -> Result<(), Error>{
    let matched = INVALID_CHARS.is_match(s);
    if !matched {
        Ok(())
    } else {
        Err(Error::from(MetricError { } ))
    }
}


#[cfg(test)]
mod tests {
    use crate::statsd_formatter::reject_invalid_chars;

    const INVALID_STRS: [&str; 5] = [
        "some|metric",
        "some#metric",
        "some,metric",
        "some:metric",
        "some=metric",
    ];

    const VALID_STR: &str = "some_str";

    #[test]
    fn test_reject_invalid_chars() {
        INVALID_STRS.iter().for_each(|invalid_str| {
            let result = reject_invalid_chars(invalid_str);
            assert!(result.is_err());
        });

        assert!(reject_invalid_chars(VALID_STR).is_ok())
    }
}