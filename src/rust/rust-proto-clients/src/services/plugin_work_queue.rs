use rust_proto::graplinc::grapl::api::plugin_work_queue::v1beta1::PluginWorkQueueServiceClient;

use crate::grpc_client_config::{
    GenericGrpcClientConfig,
    GrpcClientConfig,
};

#[derive(clap::Parser, Debug)]
pub struct PluginWorkQueueClientConfig {
    #[clap(long, env)]
    pub plugin_work_queue_client_address: String,
}

impl From<PluginWorkQueueClientConfig> for GenericGrpcClientConfig {
    fn from(val: PluginWorkQueueClientConfig) -> Self {
        GenericGrpcClientConfig {
            address: val.plugin_work_queue_client_address,
        }
    }
}

impl GrpcClientConfig for PluginWorkQueueClientConfig {
    type Client = PluginWorkQueueServiceClient;
}
