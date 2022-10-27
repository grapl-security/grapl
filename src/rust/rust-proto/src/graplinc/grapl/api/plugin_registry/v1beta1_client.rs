use bytes::Bytes;
use futures::{
    Stream,
    StreamExt,
};
use proto::plugin_registry_service_client::PluginRegistryServiceClient;
use tonic::transport::Endpoint;
use tracing::instrument;

use crate::{
    graplinc::grapl::api::{
        client::{
            Client,
            ClientError,
            Connectable,
            WithClient,
        },
        plugin_registry::v1beta1 as native,
    },
    protobufs::graplinc::grapl::api::plugin_registry::v1beta1 as proto,
};

#[async_trait::async_trait]
impl Connectable for PluginRegistryServiceClient<tonic::transport::Channel> {
    async fn connect(endpoint: Endpoint) -> Result<Self, ClientError> {
        Ok(Self::connect(endpoint).await?)
    }
}

#[derive(Clone)]
pub struct PluginRegistryClient {
    client: Client<PluginRegistryServiceClient<tonic::transport::Channel>>,
}

impl WithClient<PluginRegistryServiceClient<tonic::transport::Channel>> for PluginRegistryClient {
    fn with_client(client: Client<PluginRegistryServiceClient<tonic::transport::Channel>>) -> Self {
        Self { client }
    }
}

impl PluginRegistryClient {
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
        let proto_stream = futures::stream::iter(std::iter::once(
            native::CreatePluginRequest::Metadata(metadata),
        ))
        .chain(plugin_artifact.map(native::CreatePluginRequest::Chunk))
        .map(proto::CreatePluginRequest::from);

        self.client
            .execute_streaming(proto_stream, |mut client, request| async move {
                client.create_plugin(request).await
            })
            .await
    }

    /// retrieve the plugin corresponding to the given plugin_id
    #[instrument(skip(self, request), err)]
    pub async fn get_plugin(
        &mut self,
        request: native::GetPluginRequest,
    ) -> Result<native::GetPluginResponse, ClientError> {
        self.client
            .execute(
                request,
                |status| status.code() == tonic::Code::Unavailable,
                10,
                |mut client, request| async move { client.get_plugin(request).await },
            )
            .await
    }

    #[instrument(skip(self, request), err)]
    pub async fn list_plugins(
        &mut self,
        request: native::ListPluginsRequest,
    ) -> Result<native::ListPluginsResponse, ClientError> {
        self.client
            .execute(
                request,
                |status| status.code() == tonic::Code::Unavailable,
                10,
                |mut client, request| async move { client.list_plugins(request).await },
            )
            .await
    }

    #[instrument(skip(self, request), err)]
    pub async fn get_plugin_deployment(
        &mut self,
        request: native::GetPluginDeploymentRequest,
    ) -> Result<native::GetPluginDeploymentResponse, ClientError> {
        self.client
            .execute(
                request,
                |status| status.code() == tonic::Code::Unavailable,
                10,
                |mut client, request| async move { client.get_plugin_deployment(request).await },
            )
            .await
    }

    /// turn on a particular plugin's code
    #[instrument(skip(self, request), err)]
    pub async fn deploy_plugin(
        &mut self,
        request: native::DeployPluginRequest,
    ) -> Result<native::DeployPluginResponse, ClientError> {
        self.client
            .execute(
                request,
                |status| status.code() == tonic::Code::Unavailable,
                10,
                |mut client, request| async move { client.deploy_plugin(request).await },
            )
            .await
    }

    /// turn off a particular plugin's code
    #[instrument(skip(self, request), err)]
    pub async fn tear_down_plugin(
        &mut self,
        request: native::TearDownPluginRequest,
    ) -> Result<native::TearDownPluginResponse, ClientError> {
        self.client
            .execute(
                request,
                |status| status.code() == tonic::Code::Unavailable,
                10,
                |mut client, request| async move { client.tear_down_plugin(request).await },
            )
            .await
    }

    #[instrument(skip(self, request), err)]
    pub async fn get_plugin_health(
        &mut self,
        request: native::GetPluginHealthRequest,
    ) -> Result<native::GetPluginHealthResponse, ClientError> {
        self.client
            .execute(
                request,
                |status| status.code() == tonic::Code::Unavailable,
                10,
                |mut client, request| async move { client.get_plugin_health(request).await },
            )
            .await
    }

    /// Given information about an event source, return all generators that handle that event source
    #[tracing::instrument(skip(self, request), err)]
    pub async fn get_generators_for_event_source(
        &mut self,
        request: native::GetGeneratorsForEventSourceRequest,
    ) -> Result<native::GetGeneratorsForEventSourceResponse, ClientError> {
        self.client
            .execute(
                request,
                |status| status.code() == tonic::Code::Unavailable,
                10,
                |mut client, request| async move {
                    client.get_generators_for_event_source(request).await
                },
            )
            .await
    }

    /// Given information about a tenant, return all analyzers for that tenant
    #[instrument(skip(self, request), err)]
    pub async fn get_analyzers_for_tenant(
        &mut self,
        request: native::GetAnalyzersForTenantRequest,
    ) -> Result<native::GetAnalyzersForTenantResponse, ClientError> {
        self.client
            .execute(
                request,
                |status| status.code() == tonic::Code::Unavailable,
                10,
                |mut client, request| async move { client.get_analyzers_for_tenant(request).await },
            )
            .await
    }
}
