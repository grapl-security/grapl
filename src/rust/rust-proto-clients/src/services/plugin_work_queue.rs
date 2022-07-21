use rust_proto::graplinc::grapl::api::plugin_work_queue::v1beta1::PluginWorkQueueServiceClient;

use crate::grpc_client_config::{
    GenericGrpcClientConfig,
    GrpcClientConfig,
};

#[derive(clap::Parser, Debug)]
pub struct PluginWorkQueueClientConfig {
    #[clap(long, env)]
    pub plugin_work_queue_client_address: String,
    #[clap(long, env, default_value = crate::defaults::HEALTHCHECK_POLLING_INTERVAL_MS)]
    pub plugin_work_queue_healthcheck_polling_interval_ms: u64,
}

impl From<PluginWorkQueueClientConfig> for GenericGrpcClientConfig {
    fn from(val: PluginWorkQueueClientConfig) -> Self {
        GenericGrpcClientConfig {
            address: val.plugin_work_queue_client_address,
            healthcheck_polling_interval_ms: val.plugin_work_queue_healthcheck_polling_interval_ms,
        }
    }
}

impl GrpcClientConfig for PluginWorkQueueClientConfig {
    type Client = PluginWorkQueueServiceClient;
}
