use std::net::SocketAddr;

use structopt::StructOpt;

pub mod server;
pub mod client;

#[derive(StructOpt, Debug)]
pub struct OrganizationManagementServiceConfig {
    #[structopt(env)]
    pub organization_management_bind_address: SocketAddr,
    #[structopt(env)]
    pub organization_management_db_hostname: String,
    #[structopt(env)]
    pub organization_management_db_port: u16,
    #[structopt(env)]
    pub organization_management_db_username: String,
    #[structopt(env)]
    pub organization_management_db_password: String,
}
