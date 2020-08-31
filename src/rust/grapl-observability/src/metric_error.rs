use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum Error {
    #[error("MetricInvalidCharacterError")]
    MetricInvalidCharacterError(),
    #[error("MetricInvalidSampleRateError")]
    MetricInvalidSampleRateError(),
}