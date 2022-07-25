use std::time::Duration;

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

fn get_plugin_upstream_address(plugin_id: uuid::Uuid) -> String {
    let upstream_addr_env_var = format!("NOMAD_UPSTREAM_ADDR_plugin-{plugin_id}");
    let upstream_addr = std::env::var(&upstream_addr_env_var).expect(&upstream_addr_env_var);
    let address = format!("http://{upstream_addr}");
    address
}

/// Create a client from environment
pub async fn get_generator_client(
    plugin_id: uuid::Uuid,
) -> Result<GeneratorServiceClient, GeneratorServiceClientError> {
    let address = get_plugin_upstream_address(plugin_id);

    // TODO: Add a `rust-proto` wrapper around tonic Endpoint
    let endpoint = Endpoint::from_shared(address.to_string())?
        .timeout(Duration::from_secs(10))
        .concurrency_limit(30);

    HealthcheckClient::wait_until_healthy(
        endpoint.clone(),
        GeneratorServiceClient::SERVICE_NAME,
        Duration::from_millis(10000),
        Duration::from_millis(5000),
    )
    .await
    .expect("Generator plugin never reported healthy");

    GeneratorServiceClient::connect(endpoint).await
}
