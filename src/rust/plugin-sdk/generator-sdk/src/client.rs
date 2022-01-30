use std::{
    str::FromStr,
    time::Duration,
};

use moka::future::{
    Cache,
    CacheBuilder,
};
use rust_proto::plugin_sdk::generators::{
    generator_service_client::GeneratorServiceClient,
    GeneratorsDeserializationError,
    RunGeneratorRequest,
    RunGeneratorResponse,
};
use tonic::{
    codegen::http::uri::InvalidUri,
    transport::{
        Channel,
        ClientTlsConfig,
    },
    Code,
};
use trust_dns_resolver::{
    config::{
        NameServerConfigGroup,
        ResolverConfig,
        ResolverOpts,
    },
    error::ResolveError,
    proto::{
        error::ProtoError as ProtocolError,
        rr::rdata::SRV,
    },
    Name,
    TokioAsyncResolver,
};

use crate::{
    ClientCacheConfig,
    ClientDnsConfig,
};

#[derive(thiserror::Error, Debug)]
pub enum GeneratorClientError {
    #[error("Failed to connect to Generator {0}")]
    ConnectError(#[from] tonic::transport::Error),
    #[error("Failed to resolve name {name}")]
    EmptyResolution { name: String },
    #[error("Failed to resolve plugin {0}")]
    ResolveError(#[from] ResolveError),
    #[error("Plugin domain is invalid URI")]
    InvalidUri(#[from] InvalidUri),
    #[error(transparent)]
    Status(#[from] tonic::Status),
    #[error(transparent)]
    ProtocolError(#[from] ProtocolError),
    #[error(transparent)]
    ProtoError(#[from] GeneratorsDeserializationError),
}

type ClientCache = Cache<String, GeneratorServiceClient<Channel>>;

/// The GeneratorClient manages connections to arbitrary generators across arbitrary tenants
#[derive(Clone)]
pub struct GeneratorClient {
    /// A bounded cache mapping a plugin ID to a client connected to that plugin
    clients: ClientCache,
    /// A public certificate to validate a plugin's domain
    certificate: tonic::transport::Certificate,
    /// An in-process DNS resolver used for plugin service discovery
    resolver: TokioAsyncResolver,
}

impl GeneratorClient {
    pub fn new(
        clients: ClientCache,
        certificate: tonic::transport::Certificate,
        resolver: TokioAsyncResolver,
    ) -> Self {
        Self {
            clients,
            certificate,
            resolver,
        }
    }

    /// `run_generator` accepts arbitrary data and a plugin's identifier.
    /// We acquire a plugin connection and issue a grpc request to it.
    #[tracing::instrument(
        fields(data_len = data.len()),
        skip(self, data),
        err
    )]
    pub async fn run_generator(
        &mut self,
        data: Vec<u8>,
        plugin_id: String,
    ) -> Result<RunGeneratorResponse, GeneratorClientError> {
        let mut client = self.get_client(&plugin_id).await?;
        tracing::info!(message = "Running generator",);
        let response = client
            .run_generator(tonic::Request::new(RunGeneratorRequest { data }.into()))
            .await;
        match response {
            Ok(response) => Ok(response.into_inner().try_into()?),
            Err(status) if should_evict(&status) => {
                tracing::info!(
                    message = "Manually evicting plugin connection",
                    status = ?status,
                );
                self.clients.invalidate(&plugin_id).await;
                Err(status.into())
            }
            Err(e) => Err(e.into()),
        }
    }

    /// `get_client` attempts to grab an existing connection to a given plugin
    /// and, failing that, creates a new plugin connection
    #[tracing::instrument(skip(self))]
    async fn get_client(
        &self,
        plugin_id: &String,
    ) -> Result<GeneratorServiceClient<Channel>, GeneratorClientError> {
        match self.clients.get(plugin_id) {
            Some(client) => Ok(client),
            None => {
                let client = self.new_client_for_plugin(&plugin_id).await?;
                self.clients
                    .insert(plugin_id.to_string(), client.clone())
                    .await;
                Ok(client)
            }
        }
    }

    /// `new_client_for_plugin` creates a new gRPC client to the desired plugin.
    /// This function assumes that service discovery is against Consul and that
    /// the service's information can be queried via SRV to .service.consul.
    ///
    /// Given multiple SRV records we always choose the one with the lowest priority.
    ///
    /// We also ensure that we only connect to the plugin if it presents a valid certificate
    /// for its domain
    #[tracing::instrument(skip(self))]
    async fn new_client_for_plugin(
        &self,
        plugin_id: &String,
    ) -> Result<GeneratorServiceClient<Channel>, GeneratorClientError> {
        let domain = format!("{}.service.consul.", plugin_id);
        tracing::info!(
            message = "Resolving domain",
            domain = %domain,
        );
        let lowest_pri = self.resolve_lowest_pri(Name::from_str(&domain)?).await?;
        let tls = ClientTlsConfig::new()
            // Sets the CA Certificate against which to verify the server’s TLS certificate.
            .ca_certificate(self.certificate.clone())
            .domain_name(&domain);

        tracing::info!(
            message = "Connecting to plugin",
            target = %lowest_pri.target(),
            port = %lowest_pri.port(),
        );

        let channel = Channel::from_shared(format!(
            "https://{}:{}",
            lowest_pri.target(),
            lowest_pri.port(),
        ))?
        .tls_config(tls)?
        .connect()
        .await;

        match channel {
            Ok(channel) => Ok(GeneratorServiceClient::new(channel)),
            Err(e) => {
                // If we failed to connect we should invalidate the client from our cache
                self.clients.invalidate(plugin_id).await;
                Err(e.into())
            }
        }
    }

    /// Performs the SRV record lookup, returning the record with the lowest priority
    async fn resolve_lowest_pri(&self, name: Name) -> Result<SRV, GeneratorClientError> {
        let srvs = self.resolver.srv_lookup(name.clone()).await?;

        let lowest_priority = srvs.iter().min_by_key(|srv| srv.priority());

        match lowest_priority {
            None => Err(GeneratorClientError::EmptyResolution {
                name: name.to_string(),
            }),
            Some(lowest_priority) => Ok((*lowest_priority).clone()),
        }
    }
}

// https://github.com/grpc/grpc/blob/master/doc/statuscodes.md#status-codes-and-their-use-in-grpc
// There are three cases where we should evict the client.
// 1. If permission is denied or the service thinks we're unauthenticated this implies
//    that we have somehow connected to the wrong service (shouldn't ever happen)
// 2. If the service is unavailable. This code is raised when the server disconnects or is
//    shutting down
fn should_evict(status: &tonic::Status) -> bool {
    matches!(
        status.code(),
        Code::PermissionDenied | Code::Unauthenticated | Code::Unavailable
    )
}

impl From<ClientCacheConfig>
    for CacheBuilder<String, GeneratorServiceClient<Channel>, ClientCache>
{
    fn from(cache_config: ClientCacheConfig) -> Self {
        Cache::builder()
            .time_to_live(Duration::from_secs(cache_config.time_to_live))
            .max_capacity(cache_config.max_capacity)
    }
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
