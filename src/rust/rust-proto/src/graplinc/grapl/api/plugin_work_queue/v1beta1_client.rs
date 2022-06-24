use std::fmt::Debug;

use proto::plugin_work_queue_service_client::PluginWorkQueueServiceClient as PluginWorkQueueServiceClientProto;

use crate::{
    graplinc::grapl::api::plugin_work_queue::v1beta1 as native,
    protobufs::graplinc::grapl::api::plugin_work_queue::v1beta1 as proto,
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

impl PluginWorkQueueServiceClient {
    pub fn new(inner: PluginWorkQueueServiceClientProto<tonic::transport::Channel>) -> Self {
        Self {
            proto_client: inner,
        }
    }

    #[tracing::instrument(err)]
    pub async fn connect<T>(endpoint: T) -> Result<Self, Box<dyn std::error::Error>>
    where
        T: std::convert::TryInto<tonic::transport::Endpoint> + Debug,
        T::Error: std::error::Error + Send + Sync + 'static,
    {
        Ok(PluginWorkQueueServiceClient::new(
            PluginWorkQueueServiceClientProto::connect(endpoint).await?,
        ))
    }

    /// Adds a new execution job for a generator
    #[tracing::instrument(skip(self, request), err)]
    pub async fn put_execute_generator(
        &mut self,
        request: native::PutExecuteGeneratorRequest,
    ) -> Result<native::PutExecuteGeneratorResponse, PluginWorkQueueServiceClientError> {
        let response = self
            .proto_client
            .put_execute_generator(proto::PutExecuteGeneratorRequest::from(request))
            .await?;
        Ok(response.into_inner().try_into()?)
    }

    /// Adds a new execution job for an analyzer
    #[tracing::instrument(skip(self, request), err)]
    pub async fn put_execute_analyzer(
        &mut self,
        request: native::PutExecuteAnalyzerRequest,
    ) -> Result<native::PutExecuteAnalyzerResponse, PluginWorkQueueServiceClientError> {
        let response = self
            .proto_client
            .put_execute_analyzer(proto::PutExecuteAnalyzerRequest::from(request))
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
