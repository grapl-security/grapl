use crate::client_factory::grpc_client_config::{
    GenericGrpcClientConfig,
    GrpcClientConfig,
};

#[derive(clap::Parser, Debug)]
pub struct GeneratorClientConfig {
    // Intentionally not marked with Clap macros; you'll rarely/never have to
    // construct a GeneratorClientConfig from environment variables.
    #[clap(long, env)]
    pub generator_client_address: String,
}

impl From<GeneratorClientConfig> for GenericGrpcClientConfig {
    fn from(val: GeneratorClientConfig) -> Self {
        GenericGrpcClientConfig {
            address: val.generator_client_address,
        }
    }
}

impl GrpcClientConfig for GeneratorClientConfig {}
