use std::convert::Infallible;

use thiserror::Error;

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum ServeError {
    #[error("encountered tonic error {0}")]
    TransportError(#[from] tonic::transport::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum GrpcClientError {
    #[error("encountered protocol error {0}")]
    ErrorStatus(#[from] crate::graplinc::grapl::api::protocol::status::Status),
    #[error("encountered SerDeError {0}")]
    SerDeError(#[from] crate::SerDeError),
    #[error("circuit breaker is open")]
    CircuitOpen,
    #[error("timed out")]
    Elapsed,
}

// A compatibility layer for using
// TryFrom<Error = SerDeError>
// in place of From.
impl From<Infallible> for GrpcClientError {
    fn from(_: Infallible) -> Self {
        unreachable!()
    }
}

impl From<client_executor::Error<tonic::Status>> for GrpcClientError {
    fn from(e: client_executor::Error<tonic::Status>) -> Self {
        match e {
            client_executor::Error::Inner(e) => Self::ErrorStatus(e.into()),
            client_executor::Error::Rejected => Self::CircuitOpen,
            client_executor::Error::Elapsed => Self::Elapsed,
        }
    }
}
