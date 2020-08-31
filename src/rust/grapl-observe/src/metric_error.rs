use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum MetricError {
    #[error("MetricInvalidCharacterError")]
    MetricInvalidCharacterError(),
    #[error("MetricInvalidSampleRateError")]
    MetricInvalidSampleRateError(),
    #[error("MetricBufWriteError")]
    MetricBufWriteError(#[from] std::fmt::Error),
}
