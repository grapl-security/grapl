#[cfg(feature = "client")]
use structopt::StructOpt;

pub mod server;

#[cfg(feature = "client")]
pub mod client;

#[cfg(feature = "client")]
/// Configuration for the cache that holds onto plugin client connections
#[derive(StructOpt, Debug)]
pub struct ClientCacheConfig {
    /// The number of concurrent plugin clients to hold
    /// Defaults to 1000
    #[structopt(env, default_value = "1000")]
    pub max_capacity: u64,
    /// Total amount of time a given entry will live in seconds
    /// Default to 2 minutes
    #[structopt(env, default_value = "120")]
    pub time_to_live: u64,
}

#[cfg(feature = "client")]
use consul_connect::client_dns_config::ClientDnsConfig;

#[cfg(feature = "client")]
#[derive(StructOpt, Debug)]
pub struct ClientConfig {
    #[structopt(flatten)]
    pub client_cache_config: ClientCacheConfig,
    #[structopt(flatten)]
    pub client_dns_config: ClientDnsConfig,
}
