use std::{fmt::Debug, pin::Pin};

use futures::{Stream, StreamExt};
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

type PinnedStream<T> = Pin<Box<dyn Stream<Item = T> + Send + 'static>>;
type ResultStream<T, E> = PinnedStream<Result<T, E>>;

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

    /// create a new plugin
    pub async fn create_plugin(
        &mut self,
        request: Pin<Box<dyn Stream<Item = native::CreatePluginRequestV2> + Send>>,
    //) -> ResultStream<native::CreatePluginResponseV2, PluginRegistryServiceClientError> {
    ) -> Result<PinnedStream<native::CreatePluginResponseV2>, PluginRegistryServiceClientError> {
        // Might be nice to add a client-side "business-logic validation" hook
        // i.e. to error based on .plugin_artifact.len()
        let proto_response = self
            .proto_client
            .create_plugin(request.map(Into::into))
            .await?;
        let native_response = proto_response.into_inner().map(|result| {
            let result: Result<native::CreatePluginResponseV2, PluginRegistryServiceClientError> = {
                match result {
                    Ok(proto) => proto.try_into().map_err(Into::into),
                    Err(e) => Err(e.into()),
                }
            };
            result
        });
        Box::pin(native_response)
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
