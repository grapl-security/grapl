use sqs_executor::errors::{
    CheckedError,
    Recoverable,
};

/// Alias for a `Result` with the error type `SysmonGeneratorError`
pub type Result<T> = std::result::Result<T, SysmonGeneratorError>;

/// This represents all possible errors that can occur in this generator.
#[non_exhaustive]
#[derive(thiserror::Error, Debug)]
pub enum SysmonGeneratorError {
    /// Parsing found time value
    #[error("found negative time value: `{0}`")]
    NegativeEventTime(i64),
}

impl CheckedError for SysmonGeneratorError {
    fn error_type(&self) -> Recoverable {
        Recoverable::Persistent
    }
}
