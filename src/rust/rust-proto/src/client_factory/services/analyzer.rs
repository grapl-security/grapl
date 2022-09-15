use crate::client_factory::grpc_client_config::{
    GenericGrpcClientConfig,
    GrpcClientConfig,
};

#[derive(clap::Parser, Debug)]
pub struct AnalyzerClientConfig {
    pub analyzer_client_address: String,
}

impl From<AnalyzerClientConfig> for GenericGrpcClientConfig {
    fn from(val: AnalyzerClientConfig) -> Self {
        GenericGrpcClientConfig {
            address: val.analyzer_client_address,
        }
    }
}

impl GrpcClientConfig for AnalyzerClientConfig {}
