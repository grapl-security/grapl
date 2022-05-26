use std::net::SocketAddr;

pub mod client;
pub mod psql_queue;
pub mod server;

#[derive(clap::Parser, Debug)]
pub struct PluginWorkQueueServiceConfig {
    #[clap(long, env)]
    pub plugin_work_queue_bind_address: SocketAddr,
    #[clap(long, env)]
    pub plugin_work_queue_healthcheck_polling_interval_ms: u64,
    #[clap(long, env)]
    pub plugin_work_queue_db_hostname: String,
    #[clap(long, env)]
    pub plugin_work_queue_db_port: u16,
    #[clap(long, env)]
    pub plugin_work_queue_db_username: String,
    #[clap(long, env)]
    pub plugin_work_queue_db_password: String,
}
