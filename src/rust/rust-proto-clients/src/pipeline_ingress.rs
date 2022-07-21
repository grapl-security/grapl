use rust_proto::graplinc::grapl::api::pipeline_ingress::v1beta1::client::PipelineIngressClient;

use crate::grpc_client_config::GrpcClientConfig;

#[derive(clap::Parser, Debug)]
pub struct PipelineIngressClientConfig {
    #[clap(long, env)]
    pub pipeline_ingress_client_address: String,
    #[clap(long, env, default_value = crate::defaults::HEALTHCHECK_POLLING_INTERVAL_MS)]
    pub pipeline_ingress_healthcheck_polling_interval_ms: u64,
}

impl GrpcClientConfig for PipelineIngressClientConfig {
    type Client = PipelineIngressClient;

    fn address(&self) -> &str {
        self.pipeline_ingress_client_address.as_str()
    }
    fn healthcheck_polling_interval_ms(&self) -> u64 {
        self.pipeline_ingress_healthcheck_polling_interval_ms
    }
}
