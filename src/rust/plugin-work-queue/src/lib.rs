use std::net::SocketAddr;

pub mod client;
pub mod psql_queue;
pub mod server;
#[cfg(feature = "new_integration_tests")]
pub mod test_utils;

#[derive(clap::Parser, Clone, Debug)]
pub struct PluginWorkQueueServiceConfig {
    #[clap(long, env)]
    pub plugin_work_queue_bind_address: SocketAddr,
    #[clap(long, env)]
    pub plugin_work_queue_healthcheck_polling_interval_ms: u64,
    #[clap(flatten)]
    pub db_config: PluginWorkQueueDbConfig,
}

#[derive(clap::Parser, Clone, Debug)]
pub struct PluginWorkQueueDbConfig {
    #[clap(long, env)]
    pub plugin_work_queue_db_hostname: String,
    #[clap(long, env)]
    pub plugin_work_queue_db_port: u16,
    #[clap(long, env)]
    pub plugin_work_queue_db_username: String,
    #[clap(long, env)]
    pub plugin_work_queue_db_password: String,
}
