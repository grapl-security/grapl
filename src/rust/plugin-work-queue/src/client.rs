use async_trait;
pub use rust_proto_new::graplinc::grapl::api::plugin_work_queue::v1beta1::PluginWorkQueueServiceClient;

const ADDRESS_ENV_VAR: &'static str = "PLUGIN_WORK_QUEUE_CLIENT_ADDRESS";

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
        let address = std::env::var(ADDRESS_ENV_VAR).expect(ADDRESS_ENV_VAR);
        // TODO: introduce a rust_proto_new::Endpoint type, or pub-use the Tonic one.
        Self::connect(address.clone()).await
    }
}
