use std::fmt::Debug;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("CacheError: {0}")]
    CacheError(String),
    #[error("ProcessingError: {0}")]
    ProcessingError(String),
    #[error("OnEmissionError: {0}")]
    OnEmissionError(String),
    #[error("IoError: {0}")]
    IoError(String),
    #[error("EncodeError: {0}")]
    EncodeError(String),
    #[error("DecodeError: {0}")]
    DecodeError(String),
}
