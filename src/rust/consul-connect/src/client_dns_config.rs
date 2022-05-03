use std::time::Duration;

use structopt::StructOpt;
use trust_dns_resolver::{
    config::{
        NameServerConfigGroup,
        ResolverConfig,
        ResolverOpts,
    },
    TokioAsyncResolver,
};

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

impl From<ClientDnsConfig> for TokioAsyncResolver {
    fn from(dns_config: ClientDnsConfig) -> TokioAsyncResolver {
        let consul = ResolverConfig::from_parts(
            None,
            vec![],
            NameServerConfigGroup::from_ips_clear(
                &dns_config.dns_resolver_ips,
                dns_config.dns_resolver_port,
                true,
            ),
        );
        let opts = ResolverOpts {
            cache_size: dns_config.dns_cache_size,
            positive_min_ttl: Some(Duration::from_secs(dns_config.positive_min_ttl)),
            ..ResolverOpts::default()
        };

        TokioAsyncResolver::tokio(consul, opts).unwrap()
    }
}
