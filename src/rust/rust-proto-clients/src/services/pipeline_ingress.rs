use rust_proto::graplinc::grapl::api::pipeline_ingress::v1beta1::client::PipelineIngressClient;

use crate::grpc_client_config::{
    GenericGrpcClientConfig,
    GrpcClientConfig,
};

#[derive(clap::Parser, Debug)]
pub struct PipelineIngressClientConfig {
    #[clap(long, env)]
    pub pipeline_ingress_client_address: String,
}

impl From<PipelineIngressClientConfig> for GenericGrpcClientConfig {
    fn from(val: PipelineIngressClientConfig) -> Self {
        GenericGrpcClientConfig {
            address: val.pipeline_ingress_client_address,
        }
    }
}

impl GrpcClientConfig for PipelineIngressClientConfig {
    type Client = PipelineIngressClient;
}
