use rust_proto::graplinc::grapl::api::plugin_bootstrap::v1beta1::client::PluginBootstrapClient;

use crate::grpc_client_config::GrpcClientConfig;

#[derive(clap::Parser, Debug)]
pub struct PluginBootstrapClientConfig {
    #[clap(long, env)]
    pub plugin_bootstrap_client_address: String,
    #[clap(long, env, default_value = crate::defaults::HEALTHCHECK_POLLING_INTERVAL_MS)]
    pub plugin_bootstrap_healthcheck_polling_interval_ms: u64,
}

impl GrpcClientConfig for PluginBootstrapClientConfig {
    type Client = PluginBootstrapClient;

    fn address(&self) -> &str {
        self.plugin_bootstrap_client_address.as_str()
    }
    fn healthcheck_polling_interval_ms(&self) -> u64 {
        self.plugin_bootstrap_healthcheck_polling_interval_ms
    }
}
