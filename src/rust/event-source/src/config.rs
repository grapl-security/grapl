use std::net::SocketAddr;

use clap::Parser;

#[derive(Parser, Clone, Debug)]
pub struct EventSourceConfig {
    #[clap(flatten)]
    pub service_config: EventSourceServiceConfig,
    #[clap(flatten)]
    pub db_config: EventSourceDbConfig,
}

impl EventSourceConfig {
    /// An alias for clap::parse, so that consumers don't need to
    /// declare a dependency on clap
    pub fn from_env_vars() -> Self {
        Self::parse()
    }
}

#[derive(Parser, Clone, Debug)]
pub struct EventSourceServiceConfig {
    #[clap(long, env)]
    pub event_source_bind_address: SocketAddr,
    #[clap(long, env)]
    pub event_source_healthcheck_polling_interval_ms: u64,
}

#[derive(Parser, Clone, Debug)]
pub struct EventSourceDbConfig {
    #[clap(long, env)]
    pub event_source_db_address: String,
    #[clap(long, env)]
    pub event_source_db_username: String,
    #[clap(long, env)]
    pub event_source_db_password: grapl_config::SecretString,
}

impl grapl_config::ToPostgresUrl for EventSourceDbConfig {
    fn to_postgres_url(self) -> grapl_config::PostgresUrl {
        grapl_config::PostgresUrl {
            address: self.event_source_db_address,
            username: self.event_source_db_username,
            password: self.event_source_db_password,
        }
    }
}
