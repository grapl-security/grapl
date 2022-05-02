use std::time::Duration;

use plugin_work_queue::client::PluginWorkQueueServiceClient;
use tonic::transport::Endpoint;

#[async_trait::async_trait]
pub trait FromEnv<T, E> {
    async fn from_env() -> Result<T, E>;
}

#[async_trait::async_trait]
impl FromEnv<PluginWorkQueueServiceClient, Box<dyn std::error::Error>>
    for PluginWorkQueueServiceClient
{
    /// Create a client from environment
    async fn from_env() -> Result<PluginWorkQueueServiceClient, Box<dyn std::error::Error>> {
        const ADDRESS_ENV_VAR: &'static str = "PLUGIN_WORK_QUEUE_CLIENT_ADDRESS";
        let address = std::env::var(ADDRESS_ENV_VAR).expect(ADDRESS_ENV_VAR);
        let endpoint = Endpoint::from_shared(address)?
            .timeout(Duration::from_secs(5))
            .concurrency_limit(30);
        Self::connect(endpoint).await
    }
}
