use grapl_tracing::SetupTracingError;
use thiserror::Error;

#[non_exhaustive]
#[derive(Error, Debug)]
pub(crate) enum NodeIdentifierError {
    #[error("failed to attribute subgraph")]
    AttributionFailure,

    #[error("subgraph was empty")]
    EmptyGraph,

    #[error("error processing event {0}")]
    StreamProcessorError(#[from] kafka::StreamProcessorError),

    #[error("missing environment variable {0}")]
    MissingEnvironmentVariable(#[from] std::env::VarError),

    #[error("kafka configuration error {0}")]
    KafkaConfigurationError(#[from] kafka::ConfigurationError),

    #[error("failed to configure tracing {0}")]
    SetupTracingError(#[from] SetupTracingError),
}

impl From<NodeIdentifierError> for kafka::StreamProcessorError {
    fn from(node_identifier_error: NodeIdentifierError) -> Self {
        kafka::StreamProcessorError::EventHandlerError(node_identifier_error.to_string())
    }
}

impl From<&NodeIdentifierError> for kafka::StreamProcessorError {
    fn from(node_identifier_error: &NodeIdentifierError) -> Self {
        kafka::StreamProcessorError::EventHandlerError(node_identifier_error.to_string())
    }
}
