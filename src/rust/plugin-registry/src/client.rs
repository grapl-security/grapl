use std::time::Duration;

use async_trait;
use rust_proto_new::{
    graplinc::grapl::api::plugin_registry::v1beta1::{
        PluginRegistryServiceClient,
        PluginRegistryServiceClientError,
    },
    protocol::{
        healthcheck::client::HealthcheckClient,
        service_client::NamedService,
    },
};
use tonic::transport::Endpoint;

const ADDRESS_ENV_VAR: &'static str = "PLUGIN_REGISTRY_CLIENT_ADDRESS";

#[async_trait::async_trait]
pub trait FromEnv<T, E> {
    async fn from_env() -> Result<T, E>;
}

#[async_trait::async_trait]
impl FromEnv<PluginRegistryServiceClient, PluginRegistryServiceClientError>
    for PluginRegistryServiceClient
{
    /// Create a client from environment
    async fn from_env() -> Result<PluginRegistryServiceClient, PluginRegistryServiceClientError> {
        let address = std::env::var(ADDRESS_ENV_VAR).expect(ADDRESS_ENV_VAR);

        let endpoint = Endpoint::from_shared(address.to_string())?
            .timeout(Duration::from_secs(10))
            .concurrency_limit(30);

        HealthcheckClient::wait_until_healthy(
            endpoint.clone(),
            Self::SERVICE_NAME,
            Duration::from_millis(10000),
            Duration::from_millis(500),
        )
        .await
        .expect("plugin-registry never reported healthy");

        Self::connect(endpoint).await
    }
}
