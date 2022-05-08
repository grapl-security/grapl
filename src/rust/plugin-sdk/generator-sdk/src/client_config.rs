use structopt::StructOpt;

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

/// Configuration for the client's TLS certificate
#[derive(StructOpt, Debug)]
pub struct ClientCertConfig {
    // the CA Certificate against which to verify the server’s TLS certificate.
    #[structopt(env)]
    pub public_certificate_pem: Vec<u8>,
}

#[derive(StructOpt, Debug)]
pub struct ClientConfig {
    #[structopt(flatten)]
    pub client_cache_config: ClientCacheConfig,
    #[structopt(flatten)]
    pub client_cert_config: ClientCertConfig,
    #[structopt(flatten)]
    pub client_dns_config: consul_connect::client_dns_config::ClientDnsConfig,
}
