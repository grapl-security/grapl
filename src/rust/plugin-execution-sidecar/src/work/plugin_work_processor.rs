use kafka::Producer;
use plugin_work_queue::client::PluginWorkQueueServiceClient;
use rust_proto::{
    graplinc::grapl::{
        api::plugin_work_queue::v1beta1::{
            ExecutionJob,
            PluginWorkQueueServiceClientError,
        },
        pipeline::v1beta2::Envelope,
    },
    SerDe,
};

use crate::config::PluginExecutorConfig;

pub type RequestId = i64;

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
        config: &PluginExecutorConfig,
        pwq_client: &mut PluginWorkQueueServiceClient,
    ) -> Result<Self::Work, PluginWorkQueueServiceClientError>;

    async fn ack_work(
        &self,
        config: &PluginExecutorConfig,
        pwq_client: &mut PluginWorkQueueServiceClient,
        request_id: RequestId,
        success: bool,
    ) -> Result<(), PluginWorkQueueServiceClientError>;

    async fn process_job(
        &mut self,
        config: &PluginExecutorConfig,
        producer: &Producer<Envelope<Self::ProducedMessage>>,
        work: ExecutionJob,
    ) -> Result<(), Box<dyn std::error::Error>>;
}
