use std::net::SocketAddr;

use rust_proto::graplinc::grapl::api::plugin_sdk::generators::v1beta1::client::GeneratorServiceClient;

use crate::grpc_client_config::GrpcClientConfig;

pub struct GeneratorClientConfig {
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
