use plugin_work_queue::client::PluginWorkQueueServiceClient;
use rust_proto::graplinc::grapl::api::{
    graph::v1beta1::GraphDescription,
    plugin_sdk::generators::v1beta1::{
        client::{
            GeneratorServiceClient,
            GeneratorServiceClientError,
        },
        RunGeneratorRequest,
    },
    plugin_work_queue::v1beta1::{
        AcknowledgeGeneratorRequest,
        ExecutionJob,
        GetExecuteGeneratorRequest,
        GetExecuteGeneratorResponse,
    },
};

use super::{
    plugin_work_processor::{
        PluginWorkProcessorError,
        RequestId,
        Workload,
    },
    PluginWorkProcessor,
};
use crate::{
    config::PluginExecutorConfig,
    sidecar_client::generator_client::FromEnv,
};

impl Workload for GetExecuteGeneratorResponse {
    fn request_id(&self) -> i64 {
        self.request_id
    }
    fn maybe_job(self) -> Option<ExecutionJob> {
        self.execution_job
    }
}

pub struct GeneratorWorkProcessor {
    generator_service_client: GeneratorServiceClient,
}

impl GeneratorWorkProcessor {
    pub async fn new() -> Result<Self, GeneratorServiceClientError> {
        let generator_service_client = GeneratorServiceClient::from_env().await?;
        Ok(GeneratorWorkProcessor {
            generator_service_client,
        })
    }
}

#[async_trait::async_trait]
impl PluginWorkProcessor for GeneratorWorkProcessor {
    type Work = GetExecuteGeneratorResponse;
    type ProducedMessage = GraphDescription;

    async fn get_work(
        &self,
        config: &PluginExecutorConfig,
        pwq_client: &mut PluginWorkQueueServiceClient,
    ) -> Result<Self::Work, PluginWorkProcessorError> {
        let plugin_id = config.plugin_id;
        let get_request = GetExecuteGeneratorRequest { plugin_id };
        Ok(pwq_client.get_execute_generator(get_request).await?)
    }

    async fn ack_work(
        &self,
        config: &PluginExecutorConfig,
        pwq_client: &mut PluginWorkQueueServiceClient,
        process_result: Result<Self::ProducedMessage, PluginWorkProcessorError>,
        request_id: RequestId,
    ) -> Result<(), PluginWorkProcessorError> {
        let plugin_id = config.plugin_id;
        // TODO: Replace this with feeding the process-result back to Plugin Work Queue
        let success = process_result.is_ok();
        let ack_request = AcknowledgeGeneratorRequest {
            plugin_id,
            request_id,
            success,
        };
        pwq_client.acknowledge_generator(ack_request).await?;
        Ok(())
    }

    async fn process_job(
        &mut self,
        _config: &PluginExecutorConfig,
        job: ExecutionJob,
    ) -> Result<Self::ProducedMessage, PluginWorkProcessorError> {
        let run_generator_response = self
            .generator_service_client
            .run_generator(RunGeneratorRequest { data: job.data })
            .await?;

        Ok(run_generator_response.generated_graph.graph_description)
    }
}
