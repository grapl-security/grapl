use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum MetricError {
    #[error("InvalidCharacter")]
    InvalidCharacter(),
    #[error("InvalidSampleRate")]
    InvalidSampleRate(),
    #[error("BufWrite: {0}")]
    BufWrite(#[from] std::fmt::Error),
    #[error("IoWrite: {0}")]
    IoWrite(String),
}

impl From<std::io::Error> for MetricError {
    fn from(i: std::io::Error) -> MetricError {
        MetricError::IoWrite(i.to_string())
    }
}
