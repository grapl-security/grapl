use std::net::SocketAddr;
use structopt::StructOpt;

pub mod client;
pub mod server;
pub mod psql_queue;

#[derive(StructOpt, Debug)]
pub struct PluginWorkQueueServiceConfig {
    #[structopt(env)]
    plugin_work_queue_bind_address: SocketAddr,
    #[structopt(env)]
    plugin_work_queue_db_hostname: String,
    #[structopt(env)]
    plugin_work_queue_db_port: u16,
    #[structopt(env)]
    plugin_work_queue_db_username: String,
    #[structopt(env)]
    plugin_work_queue_db_password: String,
}
