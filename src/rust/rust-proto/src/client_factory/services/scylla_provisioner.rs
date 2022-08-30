use crate::{
    client_factory::grpc_client_config::{
        GenericGrpcClientConfig,
        GrpcClientConfig,
    },
    graplinc::grapl::api::scylla_provisioner::v1beta1::client::ScyllaProvisionerClient,
};

#[derive(clap::Parser, Debug)]
pub struct ScyllaProvisionerClientConfig {
    #[clap(long, env)]
    pub scylla_provisioner_client_address: String,
}

impl From<ScyllaProvisionerClientConfig> for GenericGrpcClientConfig {
    fn from(val: ScyllaProvisionerClientConfig) -> Self {
        GenericGrpcClientConfig {
            address: val.scylla_provisioner_client_address,
        }
    }
}

impl GrpcClientConfig for ScyllaProvisionerClientConfig {
    type Client = ScyllaProvisionerClient;
}