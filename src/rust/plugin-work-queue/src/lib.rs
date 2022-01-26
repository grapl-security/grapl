use std::net::SocketAddr;

use structopt::StructOpt;

pub mod client;
pub mod psql_queue;
pub mod server;

#[derive(StructOpt, Debug)]
pub struct PluginWorkQueueServiceConfig {
    #[structopt(env)]
    pub plugin_work_queue_bind_address: SocketAddr,
    #[structopt(env)]
    pub plugin_work_queue_db_hostname: String,
    #[structopt(env)]
    pub plugin_work_queue_db_port: u16,
    #[structopt(env)]
    pub plugin_work_queue_db_username: String,
    #[structopt(env)]
    pub plugin_work_queue_db_password: String,
}
