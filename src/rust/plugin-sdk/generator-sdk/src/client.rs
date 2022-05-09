use std::time::Duration;

use consul_connect::resolver::{
    ConsulConnectResolveError,
    ConsulConnectResolver,
};
use moka::future::{
    Cache,
    CacheBuilder,
};
use rust_proto_new::{
    graplinc::grapl::api::plugin_sdk::generators::v1beta1::{
        client::{
            GeneratorServiceClient,
            GeneratorServiceClientError,
        },
        RunGeneratorRequest,
        RunGeneratorResponse,
    },
    protocol::{
        status::{
            Code,
            Status,
        },
        tls::{
            Certificate,
            ClientTlsConfig,
        },
    },
};

use crate::client_config::{
    ClientCacheConfig,
    ClientConfig,
};

#[derive(thiserror::Error, Debug)]
pub enum GeneratorClientError {
    #[error(transparent)]
    Status(#[from] Status),
    #[error(transparent)]
    GeneratorServiceClientError(#[from] GeneratorServiceClientError),
    #[error(transparent)]
    ConsulConnectResolveError(#[from] ConsulConnectResolveError),
}

type ClientCache = Cache<String, GeneratorServiceClient>;
type ClientCacheBuilder = CacheBuilder<String, GeneratorServiceClient, ClientCache>;

/// The GeneratorClient manages connections to arbitrary generators across arbitrary tenants
#[derive(Clone)]
pub struct GeneratorClient {
    /// A bounded cache mapping a plugin ID to a client connected to that plugin
    clients: ClientCache,
    /// A public certificate to validate a plugin's domain
    certificate: Certificate,
    /// An in-process DNS resolver used for plugin service discovery
    resolver: ConsulConnectResolver,
}

impl From<ClientConfig> for GeneratorClient {
    fn from(config: ClientConfig) -> Self {
        let resolver = ConsulConnectResolver::from(config.client_dns_config);
        let certificate = Certificate::from_pem(config.client_cert_config.public_certificate_pem);
        let clients = ClientCacheBuilder::from(config.client_cache_config).build();
        Self::new(clients, certificate, resolver)
    }
}

impl GeneratorClient {
    pub fn new(
        clients: ClientCache,
        certificate: Certificate,
        resolver: ConsulConnectResolver,
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
        let mut client = self.get_client(plugin_id.clone()).await?;
        tracing::info!(message = "Running generator",);
        let response = client.run_generator(RunGeneratorRequest { data }).await;
        match response {
            Ok(response) => Ok(response),
            Err(GeneratorServiceClientError::ErrorStatus(status)) if should_evict(&status) => {
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
        plugin_id: String,
    ) -> Result<GeneratorServiceClient, GeneratorClientError> {
        match self.clients.get(&plugin_id) {
            Some(client) => Ok(client),
            None => {
                let client = self.new_client_for_plugin(plugin_id.clone()).await?;
                self.clients.insert(plugin_id, client.clone()).await;
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
        plugin_id: String,
    ) -> Result<GeneratorServiceClient, GeneratorClientError> {
        let resolved_service = self
            .resolver
            .resolve_service(format!("plugin-{plugin_id}"))
            .await?;

        // Sets the CA Certificate against which to verify the server’s TLS certificate.
        let tls_config = ClientTlsConfig::new(self.certificate.clone(), &resolved_service.domain);

        tracing::info!(
            message = "Connecting to plugin",
            target = %resolved_service.domain,
            port = %resolved_service.port,
        );

        let endpoint = format!(
            "https://{}:{}",
            resolved_service.domain, resolved_service.port,
        );

        let connect_result = GeneratorServiceClient::connect(endpoint, Some(tls_config)).await;

        match connect_result {
            Ok(x) => Ok(x),
            Err(e) => {
                // If we failed to connect we should invalidate the client from our cache
                self.clients.invalidate(&plugin_id).await;
                Err(e.into())
            }
        }
    }
}

// https://github.com/grpc/grpc/blob/master/doc/statuscodes.md#status-codes-and-their-use-in-grpc
// There are three cases where we should evict the client.
// 1. If permission is denied or the service thinks we're unauthenticated this implies
//    that we have somehow connected to the wrong service (shouldn't ever happen)
// 2. If the service is unavailable. This code is raised when the server disconnects or is
//    shutting down
fn should_evict(status: &Status) -> bool {
    matches!(
        status.code(),
        Code::PermissionDenied | Code::Unauthenticated | Code::Unavailable
    )
}

impl From<ClientCacheConfig> for ClientCacheBuilder {
    fn from(cache_config: ClientCacheConfig) -> Self {
        Cache::builder()
            .time_to_live(Duration::from_secs(cache_config.time_to_live))
            .max_capacity(cache_config.max_capacity)
    }
}
