/// Configuration for the cache that holds onto plugin client connections
#[derive(clap::Parser, Debug)]
pub struct ClientCacheConfig {
    /// The number of concurrent plugin clients to hold
    /// Defaults to 1000
    #[clap(env, default_value = "1000")]
    pub max_capacity: u64,
    /// Total amount of time a given entry will live in seconds
    /// Default to 2 minutes
    #[clap(env, default_value = "120")]
    pub time_to_live: u64,
}

/// Configuration for the client's TLS certificate
#[derive(clap::Parser, Debug)]
pub struct ClientCertConfig {
    // the CA Certificate against which to verify the server’s TLS certificate.
    #[clap(env)]
    pub public_certificate_pem: String,
}

#[derive(clap::Parser, Debug)]
pub struct ClientConfig {
    #[clap(flatten)]
    pub client_cert_config: ClientCertConfig,
    #[clap(flatten)]
    pub client_dns_config: consul_connect::client_dns_config::ClientDnsConfig,
    #[clap(flatten)]
    pub client_cache_config: ClientCacheConfig,
}
