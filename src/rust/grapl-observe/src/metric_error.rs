use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum MetricError {
    #[error("MetricInvalidCharacterError")]
    MetricInvalidCharacterError(),
    #[error("MetricInvalidSampleRateError")]
    MetricInvalidSampleRateError(),
    #[error("MetricBufWriteError: {0}")]
    MetricBufWriteError(#[from] std::fmt::Error),
    #[error("MetricIoWriteError: {0}")]
    MetricIoWriteError(String),
}

impl From<std::io::Error> for MetricError {
    fn from(i: std::io::Error) -> MetricError {
        MetricError::MetricIoWriteError(i.to_string())
    }
}
