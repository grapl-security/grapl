use core::fmt;
use std::error::Error;

#[derive(Debug)]
pub struct MetricError {
    // TODO: should this own a string or just have a reference to it?
}

impl Error for MetricError {}

impl fmt::Display for MetricError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Bad stuff with Metrics")
    }
}
