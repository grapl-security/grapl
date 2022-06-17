use std::time::Duration;

use async_trait;
use rust_proto_new::{
    graplinc::grapl::api::event_source::v1beta1::client::{
        EventSourceServiceClient,
        EventSourceServiceClientError,
    },
    protocol::{
        healthcheck::client::HealthcheckClient,
        service_client::NamedService,
    },
};

const ADDRESS_ENV_VAR: &'static str = "EVENT_SOURCE_CLIENT_ADDRESS";

#[async_trait::async_trait]
pub trait FromEnv<T, E> {
    async fn from_env() -> Result<T, E>;
}

#[async_trait::async_trait]
impl FromEnv<EventSourceServiceClient, EventSourceServiceClientError> for EventSourceServiceClient {
    /// Create a client from environment
    async fn from_env() -> Result<EventSourceServiceClient, EventSourceServiceClientError> {
        let endpoint = std::env::var(ADDRESS_ENV_VAR).expect(ADDRESS_ENV_VAR);

        HealthcheckClient::wait_until_healthy(
            endpoint.clone(),
            Self::SERVICE_NAME,
            Duration::from_millis(10000),
            Duration::from_millis(500),
        )
        .await?;

        Self::connect(endpoint, None).await
    }
}
