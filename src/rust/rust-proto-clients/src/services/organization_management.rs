use rust_proto::graplinc::grapl::api::organization_management::v1beta1::client::OrganizationManagementClient;

use crate::grpc_client_config::{
    GenericGrpcClientConfig,
    GrpcClientConfig,
};

#[derive(clap::Parser, Debug)]
pub struct OrganizationManagementClientConfig {
    #[clap(long, env)]
    pub organization_management_client_address: String,
    #[clap(long, env, default_value = crate::defaults::HEALTHCHECK_POLLING_INTERVAL_MS)]
    pub organization_management_healthcheck_polling_interval_ms: u64,
}

impl Into<GenericGrpcClientConfig> for OrganizationManagementClientConfig {
    fn into(self) -> GenericGrpcClientConfig {
        GenericGrpcClientConfig {
            address: self.organization_management_client_address,
            healthcheck_polling_interval_ms: self
                .organization_management_healthcheck_polling_interval_ms,
        }
    }
}

impl GrpcClientConfig for OrganizationManagementClientConfig {
    type Client = OrganizationManagementClient;
}
