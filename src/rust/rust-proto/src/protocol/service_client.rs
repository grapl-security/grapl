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
    #[error("Failed to connect {0}")]
    ConnectionError(#[from] tonic::transport::Error),

    #[error("Healthcheck failed for {0}: {1}")]
    HealthcheckFailed(String, HealthcheckError),

    #[error("Timeout elapsed {0}")]
    TimeoutElapsed(#[from] Elapsed),
}
