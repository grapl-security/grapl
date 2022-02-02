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
/// Configuration for the DNS resolver used for plugin service discovery
#[derive(StructOpt, Debug)]
pub struct ClientDnsConfig {
    /// IP addresses to use when resolving plugins
    /// Should almost always be pointed to Consul
    #[structopt(env)]
    pub dns_resolver_ips: Vec<std::net::IpAddr>,

    /// The port to use for DNS resolutino. Note that even if you have multiple
    /// IP addresses they will all resolve via this port
    #[structopt(env)]
    pub dns_resolver_port: u16,

    /// The number of entries in the dns cache
    #[structopt(env, default_value = "128")]
    pub dns_cache_size: usize,

    /// If this is set, any positive responses with a TTL lower than this value
    /// will have a TTL of positive_min_ttl instead.
    /// Unit: Seconds
    /// Default: 1
    #[structopt(env, default_value = "1")]
    pub positive_min_ttl: u64,
}

#[cfg(feature = "client")]
#[derive(StructOpt, Debug)]
pub struct ClientConfig {
    #[structopt(flatten)]
    pub client_cache_config: ClientCacheConfig,
    #[structopt(flatten)]
    pub client_dns_config: ClientDnsConfig,
}
