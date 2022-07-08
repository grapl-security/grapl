use std::time::Duration;

use async_trait;
use rust_proto::{
    graplinc::grapl::api::plugin_sdk::generators::v1beta1::client::{
        GeneratorServiceClient,
        GeneratorServiceClientError,
    },
    protocol::{
        healthcheck::client::HealthcheckClient,
        service_client::NamedService,
    },
};
use tonic::transport::Endpoint;

#[async_trait::async_trait]
pub trait FromEnv<T, E> {
    async fn from_env() -> Result<T, E>;
}

fn get_plugin_upstream_address_from_env() -> String {
    let plugin_id = std::env::var("PLUGIN_ID").expect("PLUGIN_ID");
    let upstream_addr_env_var = format!("NOMAD_UPSTREAM_ADDR_plugin-{plugin_id}");
    let upstream_addr = std::env::var(&upstream_addr_env_var).expect(&upstream_addr_env_var);
    let address = format!("http://{upstream_addr}");
    address
}

#[async_trait::async_trait]
impl FromEnv<GeneratorServiceClient, GeneratorServiceClientError> for GeneratorServiceClient {
    /// Create a client from environment
    async fn from_env() -> Result<GeneratorServiceClient, GeneratorServiceClientError> {
        let address = get_plugin_upstream_address_from_env();

        // TODO: Add a `rust-proto` wrapper around tonic Endpoint
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
        .expect("Generator plugin never reported healthy");

        Self::connect(endpoint).await
    }
}
