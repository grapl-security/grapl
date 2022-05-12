use std::{
    str::FromStr,
    time::Duration,
};

use trust_dns_resolver::{
    config::{
        NameServerConfigGroup,
        ResolverConfig,
        ResolverOpts,
    },
    TokioAsyncResolver,
};

/// Long story short: clap::Parser + Vec is annoying
/// Workaround inspired by https://stackoverflow.com/a/50006529
#[derive(Debug)]
pub struct CommaSeparated<T: FromStr>(Vec<T>);
impl<T: FromStr> FromStr for CommaSeparated<T> {
    type Err = <T as FromStr>::Err;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let elements: Result<Vec<_>, _> = s.split(",").map(T::from_str).collect();
        Ok(Self(elements?))
    }
}

/// Configuration for the DNS resolver used for plugin service discovery
#[derive(clap::Parser, Debug)]
pub struct ClientDnsConfig {
    /// The port to use for DNS resolutino. Note that even if you have multiple
    /// IP addresses they will all resolve via this port
    #[clap(env, long)]
    pub dns_resolver_port: u16,

    /// IP addresses to use when resolving plugins
    /// Should almost always be pointed to Consul
    #[clap(env, long)]
    pub dns_resolver_ips: CommaSeparated<std::net::IpAddr>,

    /// The number of entries in the dns cache
    #[clap(env, long, default_value = "128")]
    pub dns_cache_size: usize,

    /// If this is set, any positive responses with a TTL lower than this value
    /// will have a TTL of positive_min_ttl instead.
    /// Unit: Seconds
    /// Default: 1
    #[clap(env, long, default_value = "1")]
    pub positive_min_ttl: u64,
}

impl From<ClientDnsConfig> for TokioAsyncResolver {
    fn from(dns_config: ClientDnsConfig) -> TokioAsyncResolver {
        let CommaSeparated(ips) = dns_config.dns_resolver_ips;
        let consul = ResolverConfig::from_parts(
            None,
            vec![],
            NameServerConfigGroup::from_ips_clear(&ips, dns_config.dns_resolver_port, true),
        );
        let opts = ResolverOpts {
            cache_size: dns_config.dns_cache_size,
            positive_min_ttl: Some(Duration::from_secs(dns_config.positive_min_ttl)),
            ..ResolverOpts::default()
        };

        TokioAsyncResolver::tokio(consul, opts).unwrap()
    }
}
