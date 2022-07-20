use tokio::time::error::Elapsed;

use super::healthcheck::HealthcheckError;
use crate::protocol::endpoint::Endpoint;

/// Equivalent to the NamedService trait in tonic for server constructs.
/// Pass this NAME to e.g. a healthcheck client.
pub trait NamedService {
    const SERVICE_NAME: &'static str;
}
// TODO: Merge NamedService into Connectable

/// Every service should implement Connectable.
#[async_trait::async_trait]
pub trait Connectable {
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
}
