use rust_proto::protocol::status::Status;
use thiserror::Error;

/// This represents all possible errors that can occur in this generator.
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum SysmonGeneratorError {
    /// Parsing found time value
    #[error("found negative time value: `{0}`")]
    NegativeEventTime(i64),

    #[error("error parsing sysmon event {0}")]
    SysmonParserError(#[from] sysmon_parser::Error),

    #[error("missing environment variable {0}")]
    MissingEnvironmentVariable(#[from] std::env::VarError),

    #[error("error converting bytes to utf-8 {0}")]
    Utf8Error(#[from] std::str::Utf8Error),

    // TODO Delete when main_legacy is removed
    #[error("error processing event {0}")]
    StreamProcessorError(#[from] kafka::StreamProcessorError),

    // TODO Delete when main_legacy is removed
    #[error("kafka configuration error {0}")]
    KafkaConfigurationError(#[from] kafka::ConfigurationError),
}

// TODO Delete when main_legacy is removed
impl From<SysmonGeneratorError> for kafka::StreamProcessorError {
    fn from(val: SysmonGeneratorError) -> Self {
        kafka::StreamProcessorError::EventHandlerError(val.to_string())
    }
}

impl From<SysmonGeneratorError> for Status {
    fn from(e: SysmonGeneratorError) -> Self {
        Status::internal(e.to_string())
    }
}
