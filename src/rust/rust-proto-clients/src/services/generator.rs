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
}

impl From<GeneratorClientConfig> for GenericGrpcClientConfig {
    fn from(val: GeneratorClientConfig) -> Self {
        GenericGrpcClientConfig {
            address: val.generator_client_address,
        }
    }
}

impl GrpcClientConfig for GeneratorClientConfig {
    type Client = GeneratorServiceClient;
}