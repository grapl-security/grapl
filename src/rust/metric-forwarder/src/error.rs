use lambda_runtime::error::HandlerError;
use std::fmt::Display;
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Clone)]
pub enum MetricForwarderError {
    #[error("Couldn't create CloudwatchLogsData from gunzipped json: {0}")]
    ParseStringToLogsdataError(String),
    #[error("Couldn't base64 decode aws log data: {0}")]
    DecodeBase64Error(String),
    #[error("Couldn't gunzip decoded aws log data: {0}")]
    GunzipToStringError(String),
    #[error("Poorly formatted CloudwatchLogEvent")]
    PoorlyFormattedEventError(),
    #[error("Poorly formatted log line: {0}")]
    PoorlyFormattedLogLine(String),
    #[error("Error parsing statsd log: {0}")]
    ParseStringToStatsdError(String),
    #[error("PutMetricData to Cloudwatch error: this many failures {0}")]
    PutMetricDataError(String),
}

// can't impl From for HandlerError, sadly
pub fn to_handler_error(err: &impl Display) -> HandlerError {
    HandlerError::from(err.to_string().as_str())
}
