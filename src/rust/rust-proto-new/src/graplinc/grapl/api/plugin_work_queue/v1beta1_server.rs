use crate::{
    graplinc::grapl::api::plugin_work_queue::v1beta1 as native,
    SerDeError,
};

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum PluginWorkQueueApiError {
    #[error("failed to serialize/deserialize {0}")]
    SerDeError(#[from] SerDeError),

    #[error("received unfavorable gRPC status {0}")]
    GrpcStatus(#[from] tonic::Status),
}

pub trait Api {}

/// Implement this trait to define the API business logic
#[tonic::async_trait]
pub trait PluginWorkQueueApi<E>: Api
where
    E: Into<tonic::Status>,
{
    async fn put_execute_generator(
        &mut self,
        request: native::PutExecuteGeneratorRequest,
    ) -> Result<native::PutExecuteGeneratorResponse, E>;

    async fn put_execute_analyzer(
        &mut self,
        request: native::PutExecuteAnalyzerRequest,
    ) -> Result<native::PutExecuteAnalyzerResponse, E>;

    async fn get_execute_generator(
        &mut self,
        request: native::GetExecuteGeneratorRequest,
    ) -> Result<native::GetExecuteGeneratorResponse, E>;

    async fn get_execute_analyzer(
        &mut self,
        request: native::GetExecuteAnalyzerRequest,
    ) -> Result<native::GetExecuteAnalyzerResponse, E>;

    async fn acknowledge_generator(
        &mut self,
        request: native::AcknowledgeGeneratorRequest,
    ) -> Result<native::AcknowledgeGeneratorResponse, E>;

    async fn acknowledge_analyzer(
        &mut self,
        request: native::AcknowledgeAnalyzerRequest,
    ) -> Result<native::AcknowledgeAnalyzerResponse, E>;
}
