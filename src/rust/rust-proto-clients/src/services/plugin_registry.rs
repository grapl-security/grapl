use rust_proto::graplinc::grapl::api::plugin_registry::v1beta1::PluginRegistryServiceClient;

use crate::grpc_client_config::{
    GenericGrpcClientConfig,
    GrpcClientConfig,
};

#[derive(clap::Parser, Debug)]
pub struct PluginRegistryClientConfig {
    #[clap(long, env)]
    pub plugin_registry_client_address: String,
}

impl From<PluginRegistryClientConfig> for GenericGrpcClientConfig {
    fn from(val: PluginRegistryClientConfig) -> Self {
        GenericGrpcClientConfig {
            address: val.plugin_registry_client_address,
        }
    }
}

impl GrpcClientConfig for PluginRegistryClientConfig {
    type Client = PluginRegistryServiceClient;
}