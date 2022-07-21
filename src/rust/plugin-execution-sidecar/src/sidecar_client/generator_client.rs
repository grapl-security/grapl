use rust_proto::{
    graplinc::grapl::api::plugin_sdk::generators::v1beta1::client::GeneratorServiceClient,
    protocol::service_client::ConnectError,
};
use rust_proto_clients::{
    get_grpc_client,
    services::GeneratorClientConfig,
};

fn get_plugin_upstream_address(plugin_id: uuid::Uuid) -> String {
    let upstream_addr_env_var = format!("NOMAD_UPSTREAM_ADDR_plugin-{plugin_id}");
    let upstream_addr = std::env::var(&upstream_addr_env_var).expect(&upstream_addr_env_var);
    let address = format!("http://{upstream_addr}");
    address
}

/// Create a client from environment
pub async fn get_generator_client(
    plugin_id: uuid::Uuid,
) -> Result<GeneratorServiceClient, ConnectError> {
    let address = get_plugin_upstream_address(plugin_id);
    let client_config = GeneratorClientConfig {
        generator_client_address: address.parse().expect("generator_client_address"),
        generator_healthcheck_polling_interval_ms:
            rust_proto_clients::defaults::HEALTHCHECK_POLLING_INTERVAL_MS
                .parse()
                .expect("polling_interval_ms"),
    };
    get_grpc_client(client_config).await
}
