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

    #[error("error processing event {0}")]
    StreamProcessorError(#[from] kafka::StreamProcessorError),

    #[error("missing environment variable {0}")]
    MissingEnvironmentVariable(#[from] std::env::VarError),

    #[error("error converting bytes to utf-8 {0}")]
    Utf8Error(#[from] std::str::Utf8Error),

    #[error("kafka configuration error {0}")]
    KafkaConfigurationError(#[from] kafka::ConfigurationError),

    #[error("error configuring tracing {0}")]
    TraceError(#[from] opentelemetry::trace::TraceError),
}

impl From<SysmonGeneratorError> for Status {
    fn from(e: SysmonGeneratorError) -> Self {
        Status::internal(e.to_string())
    }
}
