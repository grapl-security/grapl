// #![allow(warnings)]
use std::net::SocketAddr;

use structopt::StructOpt;

pub mod server;
pub mod client;

#[derive(StructOpt, Debug)]
pub struct OrgManagementServiceConfig {
    #[structopt(env)]
    pub org_management_bind_address: SocketAddr,
    #[structopt(env)]
    pub org_management_db_hostname: String,
    #[structopt(env)]
    pub org_management_db_port: u16,
    #[structopt(env)]
    pub org_management_db_username: String,
    #[structopt(env)]
    pub org_management_db_password: String,
}
