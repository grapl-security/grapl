use std::net::SocketAddr;

use rust_proto::graplinc::grapl::api::plugin_bootstrap::v1beta1::client::PluginBootstrapClient;

use crate::grpc_client_config::GrpcClientConfig;

#[derive(clap::Parser, Debug)]
pub struct PluginBootstrapClientConfig {
    #[clap(long, env)]
    pub plugin_bootstrap_client_address: SocketAddr,
    #[clap(long, env, default_value = crate::defaults::HEALTHCHECK_POLLING_INTERVAL_MS)]
    pub plugin_bootstrap_healthcheck_polling_interval_ms: u64,
}

impl GrpcClientConfig for PluginBootstrapClientConfig {
    type Client = PluginBootstrapClient;

    fn address(&self) -> SocketAddr {
        self.plugin_bootstrap_client_address
    }
    fn healthcheck_polling_interval_ms(&self) -> u64 {
        self.plugin_bootstrap_healthcheck_polling_interval_ms
    }
}
