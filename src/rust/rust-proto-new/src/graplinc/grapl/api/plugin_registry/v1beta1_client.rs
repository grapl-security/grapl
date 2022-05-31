use std::{
    fmt::Debug,
    pin::Pin,
};

use futures::{
    Stream,
    StreamExt,
};
use proto::plugin_registry_service_client::PluginRegistryServiceClient as PluginRegistryServiceClientProto;

use crate::{
    graplinc::grapl::api::plugin_registry::v1beta1 as native,
    protobufs::graplinc::grapl::api::plugin_registry::v1beta1 as proto,
    SerDeError,
};

#[derive(Debug, thiserror::Error)]
pub enum PluginRegistryServiceClientError {
    #[error("ErrorStatus")]
    ErrorStatus(#[from] tonic::Status),
    #[error("PluginRegistryDeserializationError")]
    PluginRegistryDeserializationError(#[from] SerDeError),
}

pub struct PluginRegistryServiceClient {
    proto_client: PluginRegistryServiceClientProto<tonic::transport::Channel>,
}

impl PluginRegistryServiceClient {
    #[tracing::instrument(err)]
    pub async fn connect<T>(endpoint: T) -> Result<Self, Box<dyn std::error::Error>>
    where
        T: std::convert::TryInto<tonic::transport::Endpoint> + Debug,
        T::Error: std::error::Error + Send + Sync + 'static,
    {
        Ok(PluginRegistryServiceClient {
            proto_client: PluginRegistryServiceClientProto::connect(endpoint).await?,
        })
    }

    /// create a new plugin.
    /// NOTE: Most consumers will want `create_plugin`, not `create_plugin_raw`.
    pub async fn create_plugin_raw(
        &mut self,
        request: Pin<Box<dyn Stream<Item = native::CreatePluginRequestV2> + Send>>,
    ) -> Result<native::CreatePluginResponse, PluginRegistryServiceClientError> {
        let response = self
            .proto_client
            .create_plugin(request.map(proto::CreatePluginRequestV2::from))
            .await?;
        let response = native::CreatePluginResponse::try_from(response.into_inner())?;
        Ok(response)
    }

    // Simplified wrapper function for create_plugin
    pub async fn create_plugin(
        &mut self,
        metadata: native::CreatePluginRequestMetadata,
        chunks: impl Send + Iterator<Item = Vec<u8>> + 'static,
    ) -> Result<native::CreatePluginResponse, PluginRegistryServiceClientError> {
        let request = futures::stream::iter(
            vec![native::CreatePluginRequestV2::Metadata(metadata)]
        ).chain(
            futures::stream::iter(chunks.map(native::CreatePluginRequestV2::Chunk))
        );
        self.create_plugin_raw(Box::pin(request)).await
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
