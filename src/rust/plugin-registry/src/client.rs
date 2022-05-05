use std::time::Duration;

use async_trait;
use rust_proto_new::graplinc::grapl::api::plugin_registry::v1beta1::PluginRegistryServiceClient;
use tonic::transport::Endpoint;

const ADDRESS_ENV_VAR: &'static str = "PLUGIN_REGISTRY_CLIENT_ADDRESS";

#[async_trait::async_trait]
pub trait FromEnv<T, E> {
    async fn from_env() -> Result<T, E>;
}

#[async_trait::async_trait]
impl FromEnv<PluginRegistryServiceClient, Box<dyn std::error::Error>>
    for PluginRegistryServiceClient
{
    /// Create a client from environment
    async fn from_env() -> Result<PluginRegistryServiceClient, Box<dyn std::error::Error>> {
        let address = std::env::var(ADDRESS_ENV_VAR).expect(ADDRESS_ENV_VAR);
        let endpoint = Endpoint::from_shared(address)?
            .timeout(Duration::from_secs(5))
            .concurrency_limit(30);
        Self::connect(endpoint).await
    }
}
