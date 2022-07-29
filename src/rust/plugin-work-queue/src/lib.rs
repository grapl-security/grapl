use std::net::SocketAddr;

use kafka::config::ProducerConfig;

mod kafka_produce;
pub mod psql_queue;
pub mod server;
#[cfg(feature = "test-utils")]
pub mod test_utils;

// Intentionally not clap::Parser, since ProducerConfigs are
// manually constructed. Contains configs for all dependencies needed
// to construct a PluginWorkQueue.
pub struct ConfigUnion {
    pub service_config: PluginWorkQueueServiceConfig,
    pub db_config: PluginWorkQueueDbConfig,
    pub generator_producer_config: ProducerConfig,
}

#[derive(clap::Parser, Clone, Debug)]
pub struct PluginWorkQueueServiceConfig {
    #[clap(long, env)]
    pub plugin_work_queue_bind_address: SocketAddr,
    #[clap(long, env)]
    pub plugin_work_queue_healthcheck_polling_interval_ms: u64,
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
