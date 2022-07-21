use rust_proto::graplinc::grapl::api::pipeline_ingress::v1beta1::client::PipelineIngressClient;

use crate::grpc_client_config::{
    GenericGrpcClientConfig,
    GrpcClientConfig,
};

#[derive(clap::Parser, Debug)]
pub struct PipelineIngressClientConfig {
    #[clap(long, env)]
    pub pipeline_ingress_client_address: String,
    #[clap(long, env, default_value = crate::defaults::HEALTHCHECK_POLLING_INTERVAL_MS)]
    pub pipeline_ingress_healthcheck_polling_interval_ms: u64,
}

impl From<PipelineIngressClientConfig> for GenericGrpcClientConfig {
    fn from(val: PipelineIngressClientConfig) -> Self {
        GenericGrpcClientConfig {
            address: val.pipeline_ingress_client_address,
            healthcheck_polling_interval_ms: val.pipeline_ingress_healthcheck_polling_interval_ms,
        }
    }
}

impl GrpcClientConfig for PipelineIngressClientConfig {
    type Client = PipelineIngressClient;
}
