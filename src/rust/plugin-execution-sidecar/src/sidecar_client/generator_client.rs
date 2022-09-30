use clap::Parser;
use rust_proto::{
    client_factory::services::GeneratorClientConfig,
    graplinc::grapl::api::plugin_sdk::generators::v1beta1::client::GeneratorServiceClient,
    protocol::service_client::{
        ConnectError,
        ConnectWithConfig,
    },
};

/// Create a client from environment
pub async fn get_generator_client() -> Result<GeneratorServiceClient, ConnectError> {
    let client_config = GeneratorClientConfig::parse();
    GeneratorServiceClient::connect_with_config(client_config).await
}
