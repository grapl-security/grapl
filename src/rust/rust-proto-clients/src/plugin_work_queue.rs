use rust_proto::graplinc::grapl::api::plugin_work_queue::v1beta1::PluginWorkQueueServiceClient;

use crate::grpc_client_config::GrpcClientConfig;

#[derive(clap::Parser, Debug)]
pub struct PluginWorkQueueClientConfig {
    #[clap(long, env)]
    pub plugin_work_queue_client_address: String,
    #[clap(long, env, default_value = crate::defaults::HEALTHCHECK_POLLING_INTERVAL_MS)]
    pub plugin_work_queue_healthcheck_polling_interval_ms: u64,
}

impl GrpcClientConfig for PluginWorkQueueClientConfig {
    type Client = PluginWorkQueueServiceClient;

    fn address(&self) -> &str {
        self.plugin_work_queue_client_address.as_str()
    }
    fn healthcheck_polling_interval_ms(&self) -> u64 {
        self.plugin_work_queue_healthcheck_polling_interval_ms
    }
}
