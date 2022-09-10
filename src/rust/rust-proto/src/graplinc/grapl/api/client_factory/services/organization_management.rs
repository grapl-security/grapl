use crate::graplinc::grapl::api::{
    client_factory::grpc_client_config::{
        GenericGrpcClientConfig,
        GrpcClientConfig,
    },
    organization_management::v1beta1::client::OrganizationManagementClient,
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
