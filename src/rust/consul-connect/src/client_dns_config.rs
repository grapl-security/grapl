use std::{time::Duration, str::FromStr};

use structopt::StructOpt;
use trust_dns_resolver::{
    config::{
        NameServerConfigGroup,
        ResolverConfig,
        ResolverOpts,
    },
    TokioAsyncResolver,
};

/// Required due to https://stackoverflow.com/a/50006529
/// Long story short: StructOpt-with-env + Vec is annoying
#[derive(Debug)]
pub struct CommaSeparated<T: FromStr>(Vec<T>);
impl<T: FromStr> FromStr for CommaSeparated<T> {
    type Err = <T as FromStr>::Err;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let elements: Result<Vec<_>, _> = s.split(",").map(|s| {
            T::from_str(s)
        }).collect();
        Ok(Self(
            elements?
        ))
    }
}

/// Configuration for the DNS resolver used for plugin service discovery
#[derive(StructOpt, Debug)]
pub struct ClientDnsConfig {
    /// The number of entries in the dns cache
    #[structopt(env, default_value = "128")]
    pub dns_cache_size: usize,

    /// If this is set, any positive responses with a TTL lower than this value
    /// will have a TTL of positive_min_ttl instead.
    /// Unit: Seconds
    /// Default: 1
    #[structopt(env, default_value = "1")]
    pub positive_min_ttl: u64,

    /// The port to use for DNS resolutino. Note that even if you have multiple
    /// IP addresses they will all resolve via this port
    #[structopt(env)]
    pub dns_resolver_port: u16,
    
    /// IP addresses to use when resolving plugins
    /// Should almost always be pointed to Consul
    #[structopt(env)]
    pub dns_resolver_ips: CommaSeparated<std::net::IpAddr>,
}

impl From<ClientDnsConfig> for TokioAsyncResolver {
    fn from(dns_config: ClientDnsConfig) -> TokioAsyncResolver {
        let consul = ResolverConfig::from_parts(
            None,
            vec![],
            NameServerConfigGroup::from_ips_clear(
                &dns_config.dns_resolver_ips.0,
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
