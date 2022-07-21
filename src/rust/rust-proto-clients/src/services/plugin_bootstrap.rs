use rust_proto::graplinc::grapl::api::plugin_bootstrap::v1beta1::client::PluginBootstrapClient;

use crate::grpc_client_config::{
    GenericGrpcClientConfig,
    GrpcClientConfig,
};

#[derive(clap::Parser, Debug)]
pub struct PluginBootstrapClientConfig {
    #[clap(long, env)]
    pub plugin_bootstrap_client_address: String,
}

impl From<PluginBootstrapClientConfig> for GenericGrpcClientConfig {
    fn from(val: PluginBootstrapClientConfig) -> Self {
        GenericGrpcClientConfig {
            address: val.plugin_bootstrap_client_address,
        }
    }
}

impl GrpcClientConfig for PluginBootstrapClientConfig {
    type Client = PluginBootstrapClient;
}
