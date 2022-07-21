use std::fmt::Debug;

use proto::plugin_work_queue_service_client::PluginWorkQueueServiceClient as PluginWorkQueueServiceClientProto;

use crate::{
    graplinc::grapl::api::plugin_work_queue::v1beta1 as native,
    protobufs::graplinc::grapl::api::plugin_work_queue::v1beta1 as proto,
    protocol::{
        endpoint::Endpoint,
        service_client::{
            ConnectError,
            Connectable,
            NamedService,
        },
    },
    SerDeError,
};

#[derive(Debug, thiserror::Error)]
pub enum PluginWorkQueueServiceClientError {
    #[error("ErrorStatus")]
    ErrorStatus(#[from] tonic::Status),
    #[error(transparent)]
    PluginRegistryDeserializationError(#[from] SerDeError),
}

#[derive(Clone, Debug)]
pub struct PluginWorkQueueServiceClient {
    proto_client: PluginWorkQueueServiceClientProto<tonic::transport::Channel>,
}

#[async_trait::async_trait]
impl Connectable for PluginWorkQueueServiceClient {
    #[tracing::instrument(err)]
    async fn connect(endpoint: Endpoint) -> Result<Self, ConnectError> {
        Ok(Self {
            proto_client: PluginWorkQueueServiceClientProto::connect(endpoint).await?,
        })
    }
}

impl PluginWorkQueueServiceClient {
    pub fn new(inner: PluginWorkQueueServiceClientProto<tonic::transport::Channel>) -> Self {
        Self {
            proto_client: inner,
        }
    }

    /// Adds a new execution job for a generator
    #[tracing::instrument(skip(self, request), err)]
    pub async fn push_execute_generator(
        &mut self,
        request: native::PushExecuteGeneratorRequest,
    ) -> Result<native::PushExecuteGeneratorResponse, PluginWorkQueueServiceClientError> {
        let response = self
            .proto_client
            .push_execute_generator(proto::PushExecuteGeneratorRequest::from(request))
            .await?;
        Ok(response.into_inner().try_into()?)
    }

    /// Adds a new execution job for an analyzer
    #[tracing::instrument(skip(self, request), err)]
    pub async fn push_execute_analyzer(
        &mut self,
        request: native::PushExecuteAnalyzerRequest,
    ) -> Result<native::PushExecuteAnalyzerResponse, PluginWorkQueueServiceClientError> {
        let response = self
            .proto_client
            .push_execute_analyzer(proto::PushExecuteAnalyzerRequest::from(request))
            .await?;
        Ok(response.into_inner().try_into()?)
    }

    /// Retrieves a new execution job for a generator
    #[tracing::instrument(skip(self, request), err)]
    pub async fn get_execute_generator(
        &mut self,
        request: native::GetExecuteGeneratorRequest,
    ) -> Result<native::GetExecuteGeneratorResponse, PluginWorkQueueServiceClientError> {
        let response = self
            .proto_client
            .get_execute_generator(proto::GetExecuteGeneratorRequest::from(request))
            .await?;
        Ok(response.into_inner().try_into()?)
    }

    /// Retrieves a new execution job for an analyzer
    #[tracing::instrument(skip(self, request), err)]
    pub async fn get_execute_analyzer(
        &mut self,
        request: native::GetExecuteAnalyzerRequest,
    ) -> Result<native::GetExecuteAnalyzerResponse, PluginWorkQueueServiceClientError> {
        let response = self
            .proto_client
            .get_execute_analyzer(proto::GetExecuteAnalyzerRequest::from(request))
            .await?;
        Ok(response.into_inner().try_into()?)
    }

    /// Acknowledges the completion of a generator job
    #[tracing::instrument(skip(self, request), err)]
    pub async fn acknowledge_generator(
        &mut self,
        request: native::AcknowledgeGeneratorRequest,
    ) -> Result<native::AcknowledgeGeneratorResponse, PluginWorkQueueServiceClientError> {
        let response = self
            .proto_client
            .acknowledge_generator(proto::AcknowledgeGeneratorRequest::from(request))
            .await?;
        Ok(response.into_inner().try_into()?)
    }

    /// Acknowledges the completion of an analyzer job
    #[tracing::instrument(skip(self, request), err)]
    pub async fn acknowledge_analyzer(
        &mut self,
        request: native::AcknowledgeAnalyzerRequest,
    ) -> Result<native::AcknowledgeAnalyzerResponse, PluginWorkQueueServiceClientError> {
        let response = self
            .proto_client
            .acknowledge_analyzer(proto::AcknowledgeAnalyzerRequest::from(request))
            .await?;
        Ok(response.into_inner().try_into()?)
    }
}

impl NamedService for PluginWorkQueueServiceClient {
    const SERVICE_NAME: &'static str =
        "graplinc.grapl.api.plugin_work_queue.v1beta1.PluginWorkQueueService";
}
