use rust_proto::{
    graplinc::grapl::api::{
        client::ClientError,
        plugin_work_queue::v1beta1::{
            ExecutionJob,
            PluginWorkQueueClient,
        },
    },
    SerDe,
    SerDeError,
};
use uuid::Uuid;

pub type RequestId = i64;

#[derive(thiserror::Error, Debug)]
pub enum PluginWorkProcessorError {
    #[error("ClientError {0}")]
    ClientError(#[from] ClientError),

    #[error("Processing job failed, marking failed: {0}")]
    ProcessingJobFailed(String),

    #[error("Processing job failed, PWQ will retry: {0}")]
    ProcessingJobFailedRetriable(String),

    #[error("Error serializing/deserializing protocol buffer: {0}")]
    SerDeError(#[from] SerDeError),
}

impl PluginWorkProcessorError {
    pub fn is_retriable(&self) -> bool {
        match self {
            PluginWorkProcessorError::ClientError(_) => true,
            PluginWorkProcessorError::ProcessingJobFailedRetriable(_) => true,
            PluginWorkProcessorError::ProcessingJobFailed(_) => false,
            PluginWorkProcessorError::SerDeError(_) => false,
        }
    }
}

// Abstract out between Get[Generator/Analyzer]ExecutionResponse,
pub trait Workload {
    fn request_id(&self) -> RequestId;
    fn maybe_job(self) -> Option<ExecutionJob>;
}

#[async_trait::async_trait]
pub trait PluginWorkProcessor {
    type Work: Workload;
    type ProducedMessage: SerDe;

    async fn get_work(
        &self,
        plugin_id: Uuid,
        pwq_client: &mut PluginWorkQueueClient,
    ) -> Result<Self::Work, PluginWorkProcessorError>;

    async fn process_job(
        &mut self,
        plugin_id: Uuid,
        work: ExecutionJob,
    ) -> Result<Self::ProducedMessage, PluginWorkProcessorError>;

    async fn ack_work(
        &self,
        plugin_id: Uuid,
        pwq_client: &mut PluginWorkQueueClient,
        process_result: Result<Self::ProducedMessage, PluginWorkProcessorError>,
        request_id: RequestId,
        tenant_id: Uuid,
        trace_id: Uuid,
        event_source_id: Uuid,
    ) -> Result<(), PluginWorkProcessorError>;
}
