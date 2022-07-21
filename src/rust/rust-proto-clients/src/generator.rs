use std::net::SocketAddr;

use rust_proto::graplinc::grapl::api::plugin_sdk::generators::v1beta1::client::GeneratorServiceClient;

use crate::grpc_client_config::GrpcClientConfig;

#[derive(clap::Parser, Debug)]
pub struct GeneratorClientConfig {
    // Intentionally not marked with Clap macros; you'll rarely/never have to
    // construct a GeneratorClientConfig from environment variables.
    pub generator_client_address: SocketAddr,
    pub generator_healthcheck_polling_interval_ms: u64,
}

impl GrpcClientConfig for GeneratorClientConfig {
    type Client = GeneratorServiceClient;

    fn address(&self) -> SocketAddr {
        self.generator_client_address
    }
    fn healthcheck_polling_interval_ms(&self) -> u64 {
        self.generator_healthcheck_polling_interval_ms
    }
}
