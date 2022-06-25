use std::fmt::Debug;

use futures::{
    Stream,
    StreamExt,
};
use grapl_utils::iter_ext::GraplIterExt;
use proto::plugin_registry_service_client::PluginRegistryServiceClient as PluginRegistryServiceClientProto;

use crate::{
    graplinc::grapl::api::plugin_registry::v1beta1 as native,
    protobufs::graplinc::grapl::api::plugin_registry::v1beta1 as proto,
    protocol::service_client::NamedService,
    SerDeError,
};

#[derive(Debug, thiserror::Error)]
pub enum PluginRegistryServiceClientError {
    #[error("TransportError {0}")]
    TransportError(#[from] tonic::transport::Error),
    #[error("ErrorStatus {0}")]
    ErrorStatus(#[from] tonic::Status),
    #[error("PluginRegistryDeserializationError {0}")]
    PluginRegistryDeserializationError(#[from] SerDeError),
}

#[derive(Clone)]
pub struct PluginRegistryServiceClient {
    proto_client: PluginRegistryServiceClientProto<tonic::transport::Channel>,
}

impl PluginRegistryServiceClient {
    #[tracing::instrument(err)]
    pub async fn connect<T>(endpoint: T) -> Result<Self, PluginRegistryServiceClientError>
    where
        T: std::convert::TryInto<tonic::transport::Endpoint> + Debug,
        T::Error: std::error::Error + Send + Sync + 'static,
    {
        Ok(PluginRegistryServiceClient {
            proto_client: PluginRegistryServiceClientProto::connect(endpoint).await?,
        })
    }

    async fn create_analyzer_raw<S>(
        &mut self,
        request: S,
    ) -> Result<native::CreatePluginResponse, PluginRegistryServiceClientError>
    where
        S: Stream<Item = native::CreateAnalyzerRequest> + Send + 'static,
    {
        let response = self
            .proto_client
            .create_analyzer(request.map(proto::CreateAnalyzerRequest::from))
            .await?;
        let response = native::CreatePluginResponse::try_from(response.into_inner())?;
        Ok(response)
    }

    /// Create a new Analyzer plugin.
    pub async fn create_analyzer(
        &mut self,
        metadata: native::CreateAnalyzerRequestMetadata,
        plugin_artifact: impl Sized + Iterator<Item = u8> + Send + 'static,
    ) -> Result<native::CreatePluginResponse, PluginRegistryServiceClientError> {
        // Split the artifact up into 5MB chunks
        let plugin_chunks = plugin_artifact.chunks_owned(1024 * 1024 * 5);
        // Send the metadata first followed by N chunks
        let request = futures::stream::iter(std::iter::once(
            native::CreateAnalyzerRequest::Metadata(metadata),
        ))
        .chain(futures::stream::iter(
            plugin_chunks.map(native::CreateAnalyzerRequest::Chunk),
        ));
        self.create_analyzer_raw(request).await
    }

    async fn create_generator_raw<S>(
        &mut self,
        request: S,
    ) -> Result<native::CreatePluginResponse, PluginRegistryServiceClientError>
    where
        S: Stream<Item = native::CreateGeneratorRequest> + Send + 'static,
    {
        let response = self
            .proto_client
            .create_generator(request.map(proto::CreateGeneratorRequest::from))
            .await?;
        let response = native::CreatePluginResponse::try_from(response.into_inner())?;
        Ok(response)
    }

    /// Create a new Generator plugin.
    pub async fn create_generator(
        &mut self,
        metadata: native::CreateGeneratorRequestMetadata,
        plugin_artifact: impl Sized + Iterator<Item = u8> + Send + 'static,
    ) -> Result<native::CreatePluginResponse, PluginRegistryServiceClientError> {
        // Split the artifact up into 5MB chunks
        let plugin_chunks = plugin_artifact.chunks_owned(1024 * 1024 * 5);
        // Send the metadata first followed by N chunks
        let request = futures::stream::iter(std::iter::once(
            native::CreateGeneratorRequest::Metadata(metadata),
        ))
        .chain(futures::stream::iter(
            plugin_chunks.map(native::CreateGeneratorRequest::Chunk),
        ));
        self.create_generator_raw(request).await
    }

    /// retrieve the plugin corresponding to the given plugin_id
    pub async fn get_plugin(
        &mut self,
        request: native::GetPluginRequest,
    ) -> Result<native::GetPluginResponse, PluginRegistryServiceClientError> {
        let response = self
            .proto_client
            .get_plugin(proto::GetPluginRequest::from(request))
            .await?;
        let response = native::GetPluginResponse::try_from(response.into_inner())?;
        Ok(response)
    }

    /// turn on a particular plugin's code
    pub async fn deploy_plugin(
        &mut self,
        request: native::DeployPluginRequest,
    ) -> Result<native::DeployPluginResponse, PluginRegistryServiceClientError> {
        let response = self
            .proto_client
            .deploy_plugin(proto::DeployPluginRequest::from(request))
            .await?;
        let response = native::DeployPluginResponse::try_from(response.into_inner())?;
        Ok(response)
    }

    /// turn off a particular plugin's code
    pub async fn tear_down_plugin(
        &mut self,
        request: native::TearDownPluginRequest,
    ) -> Result<native::TearDownPluginResponse, PluginRegistryServiceClientError> {
        self.proto_client
            .tear_down_plugin(proto::TearDownPluginRequest::from(request))
            .await?;
        todo!()
    }

    /// Given information about an event source, return all generators that handle that event source
    #[tracing::instrument(skip(self, request), err)]
    pub async fn get_generators_for_event_source(
        &mut self,
        request: native::GetGeneratorsForEventSourceRequest,
    ) -> Result<native::GetGeneratorsForEventSourceResponse, PluginRegistryServiceClientError> {
        self.proto_client
            .get_generators_for_event_source(proto::GetGeneratorsForEventSourceRequest::from(
                request,
            ))
            .await?;
        todo!()
    }

    /// Given information about a tenant, return all analyzers for that tenant
    pub async fn get_analyzers_for_tenant(
        &mut self,
        request: native::GetAnalyzersForTenantRequest,
    ) -> Result<native::GetAnalyzersForTenantResponse, PluginRegistryServiceClientError> {
        self.proto_client
            .get_analyzers_for_tenant(proto::GetAnalyzersForTenantRequest::from(request))
            .await?;
        todo!()
    }
}

impl NamedService for PluginRegistryServiceClient {
    const SERVICE_NAME: &'static str =
        "graplinc.grapl.api.plugin_registry.v1beta1.PluginRegistryService";
}
