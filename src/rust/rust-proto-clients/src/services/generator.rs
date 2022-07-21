use rust_proto::graplinc::grapl::api::plugin_sdk::generators::v1beta1::client::GeneratorServiceClient;

use crate::grpc_client_config::{
    GenericGrpcClientConfig,
    GrpcClientConfig,
};

#[derive(clap::Parser, Debug)]
pub struct GeneratorClientConfig {
    // Intentionally not marked with Clap macros; you'll rarely/never have to
    // construct a GeneratorClientConfig from environment variables.
    pub generator_client_address: String,
    pub generator_healthcheck_polling_interval_ms: u64,
}

impl Into<GenericGrpcClientConfig> for GeneratorClientConfig {
    fn into(self) -> GenericGrpcClientConfig {
        GenericGrpcClientConfig {
            address: self.generator_client_address,
            healthcheck_polling_interval_ms: self.generator_healthcheck_polling_interval_ms,
        }
    }
}

impl GrpcClientConfig for GeneratorClientConfig {
    type Client = GeneratorServiceClient;
}
