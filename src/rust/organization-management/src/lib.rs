use std::net::SocketAddr;

pub mod server;

#[derive(clap::Parser, Debug)]
pub struct OrganizationManagementServiceConfig {
    #[clap(long, env)]
    pub organization_management_bind_address: SocketAddr,
    #[clap(long, env)]
    pub organization_management_healthcheck_polling_interval_ms: u64,
    #[clap(long, env)]
    pub organization_management_db_address: String,
    #[clap(long, env)]
    pub organization_management_db_username: String,
    #[clap(long, env)]
    pub organization_management_db_password: grapl_config::SecretString,
}

impl grapl_config::ToPostgresUrl for OrganizationManagementServiceConfig {
    fn to_postgres_url(self) -> grapl_config::PostgresUrl {
        grapl_config::PostgresUrl {
            address: self.organization_management_db_address,
            username: self.organization_management_db_username,
            password: self.organization_management_db_password,
        }
    }
}
