use rust_proto::graplinc::grapl::api::organization_management::v1beta1::client::OrganizationManagementClient;

use crate::grpc_client_config::GrpcClientConfig;

#[derive(clap::Parser, Debug)]
pub struct OrganizationManagementClientConfig {
    #[clap(long, env)]
    pub organization_management_client_address: String,
    #[clap(long, env, default_value = crate::defaults::HEALTHCHECK_POLLING_INTERVAL_MS)]
    pub organization_management_healthcheck_polling_interval_ms: u64,
}

impl GrpcClientConfig for OrganizationManagementClientConfig {
    type Client = OrganizationManagementClient;

    fn address(&self) -> &str {
        self.organization_management_client_address.as_str()
    }
    fn healthcheck_polling_interval_ms(&self) -> u64 {
        self.organization_management_healthcheck_polling_interval_ms
    }
}
