use std::time::Duration;

use plugin_work_queue::client::PluginWorkQueueServiceClient;
use tonic::transport::Endpoint;

pub async fn plugin_work_queue_client_from_env() -> Result<PluginWorkQueueServiceClient, Box<dyn std::error::Error>> {
    const ADDRESS_ENV_VAR: &'static str = "PLUGIN_WORK_QUEUE_CLIENT_ADDRESS";
    let address = std::env::var(ADDRESS_ENV_VAR).expect(ADDRESS_ENV_VAR);
    let endpoint = Endpoint::from_shared(address)?
        .timeout(Duration::from_secs(5))
        .concurrency_limit(30);
    PluginWorkQueueServiceClient::connect(endpoint).await
}