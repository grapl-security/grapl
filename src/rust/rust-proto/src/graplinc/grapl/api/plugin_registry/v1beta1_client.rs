use std::time::Duration;

use bytes::Bytes;
use client_executor::strategy::FibonacciBackoff;
use futures::{
    Stream,
    StreamExt,
};
use proto::plugin_registry_service_client::PluginRegistryServiceClient;
use tonic::transport::Endpoint;
use tracing::instrument;

use crate::{
    graplinc::grapl::api::{
        plugin_registry::v1beta1 as native,
        client::{
            Connectable,
            Client,
            ClientError,
            Configuration
        },
    },
    protobufs::graplinc::grapl::api::plugin_registry::v1beta1 as proto,
};

#[async_trait::async_trait]
impl Connectable
    for PluginRegistryServiceClient<tonic::transport::Channel>
{
    async fn connect(endpoint: Endpoint) -> Result<Self, ClientError> {
        Ok(Self::connect(endpoint).await?)
    }
}

#[derive(Clone)]
pub struct PluginRegistryClient<B>
where
    B: IntoIterator<Item = Duration> + Clone,
{
    client: Client<B, PluginRegistryServiceClient<tonic::transport::Channel>>,
}

impl <B> PluginRegistryClient<B>
where
    B: IntoIterator<Item = Duration> + Clone,
{
    const SERVICE_NAME: &'static str =
        "graplinc.grapl.api.plugin_registry.v1beta1.PluginRegistryService";

    pub fn new<A>(
        address: A,
        request_timeout: Duration,
        executor_timeout: Duration,
        concurrency_limit: usize,
        initial_backoff_delay: Duration,
        maximum_backoff_delay: Duration,
    ) -> Result<Self, ClientError>
    where
        A: TryInto<Endpoint>,
    {
        let configuration = Configuration::new(
            Self::SERVICE_NAME,
            address,
            request_timeout,
            executor_timeout,
            concurrency_limit,
            FibonacciBackoff::from_millis(initial_backoff_delay.as_millis())
                .max_delay(maximum_backoff_delay)
                .map(client_executor::strategy::jitter),
        )?;
        let client = Client::new(configuration)?;

        Ok(Self { client })
    }

    /// Create a new plugin
    #[instrument(skip(self, metadata, plugin_artifact), err)]
    pub async fn create_plugin<S>(
        &mut self,
        metadata: native::PluginMetadata,
        plugin_artifact: S,
    ) -> Result<native::CreatePluginResponse, ClientError>
    where
        S: Stream<Item = Bytes> + Send + 'static,
    {
        // Send the metadata first followed by N chunks
        let request = futures::stream::iter(std::iter::once(
            native::CreatePluginRequest::Metadata(metadata),
        ))
        .chain(plugin_artifact.map(
            native::CreatePluginRequest::Chunk
        ));

        self.client.execute_streaming(
            request,
            |status| status.code() == tonic::Code::Unavailable,
            10,
            |client, request| client.create_plugin(request),
        ).await?
    }

    /// retrieve the plugin corresponding to the given plugin_id
    #[instrument(skip(self, request), err)]
    pub async fn get_plugin(
        &mut self,
        request: native::GetPluginRequest,
    ) -> Result<native::GetPluginResponse, ClientError> {
        Ok(self.client.execute(
            request,
            |status| status.code() == tonic::Code::Unavailable,
            10,
            |client, request| client.get_plugin(request)
        ).await?)
    }

    #[instrument(skip(self, request), err)]
    pub async fn list_plugins(
        &mut self,
        request: native::ListPluginsRequest,
    ) -> Result<native::ListPluginsResponse, ClientError> {
        Ok(self.client.execute(
            request,
            |status, request| status.code() == tonic::Code::Unavailable,
            10,
            |client, request| client.list_plugins(request)
        ).await?)
    }

    #[instrument(skip(self, request), err)]
    pub async fn get_plugin_deployment(
        &mut self,
        request: native::GetPluginDeploymentRequest,
    ) -> Result<native::GetPluginDeploymentResponse, ClientError> {
        Ok(self.client.execute(
            request,
            |status| status.code() == tonic::Code::Unavailable,
            10,
            |client, request| client.get_plugin_deployment(request)
        ).await?)
    }

    /// turn on a particular plugin's code
    #[instrument(skip(self, request), err)]
    pub async fn deploy_plugin(
        &mut self,
        request: native::DeployPluginRequest,
    ) -> Result<native::DeployPluginResponse, ClientError> {
        Ok(self.client.execute(
            request,
            |status| status.code() = tonic::Code::Unavailable,
            10,
            |client, request| client.deploy_plugin(request)
        ).await?)
    }

    /// turn off a particular plugin's code
    #[instrument(skip(self, request), err)]
    pub async fn tear_down_plugin(
        &mut self,
        request: native::TearDownPluginRequest,
    ) -> Result<native::TearDownPluginResponse, ClientError> {
        Ok(self.client.execute(
            request,
            |status| status.code() == tonic::Code::Unavailable,
            10,
            |client, request| client.tear_down_plugin(request)
        ).await?)
    }

    #[instrument(skip(self, request), err)]
    pub async fn get_plugin_health(
        &mut self,
        request: native::GetPluginHealthRequest,
    ) -> Result<native::GetPluginHealthResponse, ClientError> {
        Ok(self.client.execute(
            request,
            |status| status.code() == tonic::Code::Unavailable,
            10,
            |client, request| client.get_plugin_health(request)
        ).await?)
    }

    /// Given information about an event source, return all generators that handle that event source
    #[tracing::instrument(skip(self, request), err)]
    pub async fn get_generators_for_event_source(
        &mut self,
        request: native::GetGeneratorsForEventSourceRequest,
    ) -> Result<native::GetGeneratorsForEventSourceResponse, ClientError> {
        Ok(self.client.execute(
            request,
            |status| status.code() == tonic::Code::Unavailable,
            10,
            |client, request| client.get_generators_for_event_source(request)
        ).await?)
    }

    /// Given information about a tenant, return all analyzers for that tenant
    #[instrument(skip(self, request), err)]
    pub async fn get_analyzers_for_tenant(
        &mut self,
        request: native::GetAnalyzersForTenantRequest,
    ) -> Result<native::GetAnalyzersForTenantResponse, ClientError> {
        Ok(self.client.execute(
            request,
            |status| status.code() == tonic::Code::Unavailable,
            10,
            |client, request| client.get_analyzers_for_tenant(request)
        ).await?)
    }
}
