use std::{
    collections::HashMap,
    str::FromStr,
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
};
use trust_dns_resolver::{
    config::{
        NameServerConfigGroup,
        ResolverConfig,
        ResolverOpts,
    },
    error::ResolveError,
    Name,
    TokioAsyncResolver,
};

#[derive(thiserror::Error, Debug)]
pub enum GeneratorClientError {
    #[error("Failed to connect to Generator {0}")]
    ConnectError(#[from] tonic::transport::Error),
    #[error("Failed to resolve generator {plugin_name}")]
    EmptyResolution { plugin_name: String },
    #[error("Failed to resolve plugin {0}")]
    ResolveError(#[from] ResolveError),
    #[error("Plugin domain is invalid URI")]
    InvalidUri(#[from] InvalidUri),
    #[error(transparent)]
    Status(#[from] tonic::Status),
    #[error(transparent)]
    ProtoError(#[from] GeneratorsDeserializationError),
}

#[derive(Clone)]
pub struct GeneratorClient {
    clients: HashMap<String, GeneratorServiceClient<Channel>>,
    certificate: tonic::transport::Certificate,
    resolver: TokioAsyncResolver,
}

impl GeneratorClient {
    // `run_generator` takes a `plugin_name` and `data`. It resolves the `plugin_name` to an address
    // and calls the grpc `run_generator` on that address, supplying the data to it.
    pub async fn run_generator(
        &mut self,
        data: Vec<u8>,
        plugin_name: String,
    ) -> Result<RunGeneratorResponse, GeneratorClientError> {
        let mut client = self.get_client(plugin_name).await?;
        let response = client
            .run_generator(tonic::Request::new(RunGeneratorRequest { data }.into()))
            .await?;
        Ok(response.into_inner().try_into()?)
    }

    // `get_client` attempts to grab an existing connection to a given plugin
    // and, failing that, creates a new plugin connection
    async fn get_client(
        &mut self,
        plugin_name: String,
    ) -> Result<GeneratorServiceClient<Channel>, GeneratorClientError> {
        match self.clients.get(&plugin_name).clone() {
            Some(client) => {
                let client = client.clone();
                Ok(client)
            }
            None => {
                let client = self.new_client_for_plugin(&plugin_name).await?;
                self.clients.insert(plugin_name.to_string(), client.clone());
                Ok(client)
            }
        }
    }

    async fn new_client_for_plugin(
        &self,
        plugin_name: &str,
    ) -> Result<GeneratorServiceClient<Channel>, GeneratorClientError> {
        let domain = format!("{}.service.consul.", plugin_name);
        let srvs = self
            .resolver
            .srv_lookup(Name::from_str(&domain).unwrap())
            .await?;

        let mut srvs: Vec<_> = srvs.iter().collect();

        if srvs.is_empty() {
            return Err(GeneratorClientError::EmptyResolution {
                plugin_name: plugin_name.to_string(),
            });
        }
        srvs.sort_unstable_by_key(|srv| srv.priority());
        let lowest_pri = srvs.last().unwrap(); // already checked - not empty

        let tls = ClientTlsConfig::new()
            // Sets the CA Certificate against which to verify the server’s TLS certificate.
            .ca_certificate(self.certificate.clone())
            .domain_name(&domain);

        let channel = Channel::from_shared(format!(
            "https://{}:{}",
            lowest_pri.target(),
            lowest_pri.port()
        ))?
        .tls_config(tls)?
        .connect()
        .await?;

        Ok(GeneratorServiceClient::new(channel))
    }
}

pub fn default_resolver() -> TokioAsyncResolver {
    let consul = ResolverConfig::from_parts(
        None,
        vec![],
        NameServerConfigGroup::from_ips_clear(&["127.0.0.1".parse().unwrap()], 8500, true),
    );
    TokioAsyncResolver::tokio(consul, ResolverOpts::default()).unwrap()
}
