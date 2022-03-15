use sqs_executor::{
    errors::{
        CheckedError,
        Recoverable,
    },
};

/// This represents all possible errors that can occur in this generator.
#[non_exhaustive]
#[derive(thiserror::Error, Debug)]
pub enum SysmonGeneratorError {
    /// Parsing found time value
    #[error("found negative time value: `{0}`")]
    NegativeEventTime(i64),

    /// Unable to parse datetime
    #[error("datetime parse error: `{0}`")]
    TimeError(#[from] chrono::ParseError),

    /// TODO(inickles) this shouldn't be an error, it's expected. Remove this.
    #[error("Unsupported event type")]
    UnsupportedEventType(String),
}

impl CheckedError for SysmonGeneratorError {
    fn error_type(&self) -> Recoverable {
        Recoverable::Persistent
    }
}