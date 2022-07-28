use tokio::time::error::Elapsed;

use super::healthcheck::HealthcheckError;
use crate::protocol::endpoint::Endpoint;

/// Every service should implement Connectable.
#[async_trait::async_trait]
pub trait Connectable {
    /// Pass this NAME to e.g. a healthcheck client.
    const SERVICE_NAME: &'static str;

    async fn connect(endpoint: Endpoint) -> Result<Self, ConnectError>
    where
        Self: Sized;
}

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum ConnectError {
    #[error("failed to connect {0}")]
    ConnectionError(#[from] tonic::transport::Error),

    #[error("healthcheck failed {0}")]
    HealtcheckFailed(#[from] HealthcheckError),

    #[error("timeout elapsed {0}")]
    TimeoutElapsed(#[from] Elapsed),

    #[error("Circuit Breaker is open")]
    CircuitBreakerOpen,
}

impl From<client_executor::Error<ConnectError>> for ConnectError {
    fn from(e: client_executor::Error<ConnectError>) -> Self {
        match e {
            client_executor::Error::Inner(e) => e,
            client_executor::Error::Rejected => ConnectError::CircuitBreakerOpen,
            client_executor::Error::Elapsed(e) => ConnectError::TimeoutElapsed(e),
        }
    }
}
