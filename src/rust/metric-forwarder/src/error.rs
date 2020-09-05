use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum MetricForwarderError {
    #[error("Couldn't create CloudwatchLogsData from gunzipped json: {0}")]
    ParseStringToLogsdataError(String),
    #[error("Couldn't base64 decode aws log data: {0}")]
    DecodeBase64Error(String),
    #[error("Couldn't gunzip decoded aws log data: {0}")]
    GunzipToStringError(String),
    #[error("Poorly formatted CloudwatchLogEvent")]
    PoorlyFormattedEventError(),
}
