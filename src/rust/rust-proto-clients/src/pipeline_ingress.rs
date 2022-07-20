use std::net::SocketAddr;

use rust_proto::graplinc::grapl::api::pipeline_ingress::v1beta1::client::PipelineIngressClient;

use crate::grpc_client_config::GrpcClientConfig;

#[derive(clap::Parser, Debug)]
pub struct PipelineIngressClientConfig {
    #[clap(long, env)]
    pub pipeline_ingress_client_address: SocketAddr,
    #[clap(long, env, default_value = crate::defaults::HEALTHCHECK_POLLING_INTERVAL_MS)]
    pub pipeline_ingress_healthcheck_polling_interval_ms: u64,
}

impl GrpcClientConfig for PipelineIngressClientConfig {
    type Client = PipelineIngressClient;

    fn address(&self) -> SocketAddr {
        self.pipeline_ingress_client_address
    }
    fn healthcheck_polling_interval_ms(&self) -> u64 {
        self.pipeline_ingress_healthcheck_polling_interval_ms
    }
}
