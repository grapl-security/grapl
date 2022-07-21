use rust_proto::graplinc::grapl::api::organization_management::v1beta1::client::OrganizationManagementClient;

use crate::grpc_client_config::{
    GenericGrpcClientConfig,
    GrpcClientConfig,
};

#[derive(clap::Parser, Debug)]
pub struct OrganizationManagementClientConfig {
    #[clap(long, env)]
    pub organization_management_client_address: String,
}

impl From<OrganizationManagementClientConfig> for GenericGrpcClientConfig {
    fn from(val: OrganizationManagementClientConfig) -> Self {
        GenericGrpcClientConfig {
            address: val.organization_management_client_address,
        }
    }
}

impl GrpcClientConfig for OrganizationManagementClientConfig {
    type Client = OrganizationManagementClient;
}
