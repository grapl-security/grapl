use std::time::Duration;

use figment::{
    providers::Env,
    Figment,
};
use rust_proto::graplinc::grapl::api::{
    client::{
        ClientConfiguration,
        Connect,
    },
    graph::v1beta1::GraphDescription,
    plugin_sdk::generators::v1beta1::{
        client::GeneratorClient,
        RunGeneratorRequest,
    },
    plugin_work_queue::v1beta1::{
        AcknowledgeGeneratorRequest,
        ExecutionJob,
        GetExecuteGeneratorRequest,
        GetExecuteGeneratorResponse,
        PluginWorkQueueClient,
    },
};
use uuid::Uuid;

use super::{
    plugin_work_processor::{
        PluginWorkProcessorError,
        RequestId,
        Workload,
    },
    PluginWorkProcessor,
};

impl Workload for GetExecuteGeneratorResponse {
    fn request_id(&self) -> i64 {
        self.request_id()
    }

    fn maybe_job(self) -> Option<ExecutionJob> {
        self.execution_job()
    }
}

pub struct GeneratorWorkProcessor {
    generator_client: GeneratorClient,
}

impl GeneratorWorkProcessor {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let client_config: ClientConfiguration = Figment::new()
            .merge(Env::prefixed("GENERATOR_"))
            .extract()?;

        let generator_client = GeneratorClient::connect_with_healthcheck(
            client_config,
            Duration::from_secs(60),
            Duration::from_secs(1),
        )
        .await?;

        Ok(GeneratorWorkProcessor { generator_client })
    }
}

#[async_trait::async_trait]
impl PluginWorkProcessor for GeneratorWorkProcessor {
    type Work = GetExecuteGeneratorResponse;
    type ProducedMessage = GraphDescription;

    async fn get_work(
        &self,
        plugin_id: Uuid,
        pwq_client: &mut PluginWorkQueueClient,
    ) -> Result<Self::Work, PluginWorkProcessorError> {
        let response = pwq_client
            .get_execute_generator(GetExecuteGeneratorRequest::new(plugin_id))
            .await?;
        let request_id = response.request_id();

        let response_retval = response.clone();

        if let Some(execution_job) = response.execution_job() {
            let tenant_id = execution_job.tenant_id();
            let trace_id = execution_job.trace_id();
            let event_source_id = execution_job.event_source_id();

            tracing::debug!(
                message = "retrieved execution job",
                tenant_id =% tenant_id,
                trace_id =% trace_id,
                event_source_id =% event_source_id,
                plugin_id =% plugin_id,
                request_id =? request_id,
            );
        } else {
            tracing::debug!(
                message = "found no execution jobs",
                plugin_id =% plugin_id,
            );
        }

        Ok(response_retval)
    }

    async fn ack_work(
        &self,
        plugin_id: Uuid,
        pwq_client: &mut PluginWorkQueueClient,
        process_result: Result<Self::ProducedMessage, PluginWorkProcessorError>,
        request_id: RequestId,
        tenant_id: Uuid,
        trace_id: Uuid,
        event_source_id: Uuid,
    ) -> Result<(), PluginWorkProcessorError> {
        tracing::debug!(
            message = "acknowledging generator work",
            tenant_id =% tenant_id,
            trace_id =% trace_id,
            event_source_id =% event_source_id,
            plugin_id =% plugin_id,
            request_id =? request_id,
        );

        let graph_description = process_result.ok();
        let ack_request = AcknowledgeGeneratorRequest::new(
            request_id,
            graph_description,
            plugin_id,
            tenant_id,
            trace_id,
            event_source_id,
        );
        pwq_client.acknowledge_generator(ack_request).await?;
        Ok(())
    }

    async fn process_job(
        &mut self,
        _plugin_id: Uuid,
        job: ExecutionJob,
    ) -> Result<Self::ProducedMessage, PluginWorkProcessorError> {
        let run_generator_response = self
            .generator_client
            .run_generator(RunGeneratorRequest {
                data: job.data().clone(),
            })
            .await?;

        Ok(run_generator_response.generated_graph.graph_description)
    }
}
