use tokio::time::error::Elapsed;

use super::healthcheck::HealthcheckError;
use crate::{
    client_factory::{
        build_grpc_client,
        grpc_client_config::GrpcClientConfig,
    },
    protocol::endpoint::Endpoint,
};

/// Every service should implement Connectable.
#[async_trait::async_trait]
pub trait Connectable {
    type Config: GrpcClientConfig + Send + Sync;

    /// Pass this NAME to e.g. a healthcheck client.
    const SERVICE_NAME: &'static str;

    async fn connect_with_endpoint(endpoint: Endpoint) -> Result<Self, ConnectError>
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

    #[error("Timeout elapsed")]
    TimeoutElapsed,

    #[error("CircuitBreaker Open")]
    CircuitBreakerOpen,
}

impl From<Elapsed> for ConnectError {
    fn from(_e: Elapsed) -> Self {
        Self::TimeoutElapsed
    }
}

impl From<client_executor::Error<ConnectError>> for ConnectError {
    fn from(e: client_executor::Error<ConnectError>) -> Self {
        match e {
            client_executor::Error::Inner(e) => e,
            client_executor::Error::Rejected => Self::CircuitBreakerOpen,
            client_executor::Error::Elapsed => Self::TimeoutElapsed,
        }
    }
}

#[async_trait::async_trait]
pub trait ConnectWithConfig
where
    Self: Connectable,
{
    async fn connect_with_config(client_config: Self::Config) -> Result<Self, ConnectError>
    where
        Self: Sized;
}

#[async_trait::async_trait]
impl<T> ConnectWithConfig for T
where
    T: Connectable,
{
    async fn connect_with_config(client_config: Self::Config) -> Result<Self, ConnectError> {
        build_grpc_client(client_config).await
    }
}
