use std::net::SocketAddr;

pub mod client;
pub mod server;

#[derive(clap::Parser, Debug)]
pub struct OrganizationManagementServiceConfig {
    #[clap(long, env)]
    pub organization_management_bind_address: SocketAddr,
    #[clap(long, env)]
    pub organization_management_db_hostname: String,
    #[clap(long, env)]
    pub organization_management_db_port: u16,
    #[clap(long, env)]
    pub organization_management_db_username: String,
    #[clap(long, env)]
    pub organization_management_db_password: String,
}
