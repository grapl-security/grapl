use rust_proto::graplinc::grapl::api::plugin_registry::v1beta1::PluginRegistryServiceClient;

use crate::grpc_client_config::{
    GenericGrpcClientConfig,
    GrpcClientConfig,
};

#[derive(clap::Parser, Debug)]
pub struct PluginRegistryClientConfig {
    #[clap(long, env)]
    pub plugin_registry_client_address: String,
    #[clap(long, env, default_value = crate::defaults::HEALTHCHECK_POLLING_INTERVAL_MS)]
    pub plugin_registry_healthcheck_polling_interval_ms: u64,
}

impl Into<GenericGrpcClientConfig> for PluginRegistryClientConfig {
    fn into(self) -> GenericGrpcClientConfig {
        GenericGrpcClientConfig {
            address: self.plugin_registry_client_address,
            healthcheck_polling_interval_ms: self.plugin_registry_healthcheck_polling_interval_ms,
        }
    }
}

impl GrpcClientConfig for PluginRegistryClientConfig {
    type Client = PluginRegistryServiceClient;
}
