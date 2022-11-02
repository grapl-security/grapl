use figment::{
    providers::{
        Env,
        Format,
        Json,
    },
    Figment,
};
use rust_proto::{
    graplinc::grapl::api::{
        client::{
            ClientConfiguration,
            Connect,
        },
        plugin_sdk::analyzers::v1beta1::{
            client::AnalyzerClient,
            messages::{
                ExecutionResult,
                RunAnalyzerRequest,
                Update,
            },
        },
        plugin_work_queue::v1beta1::{
            AcknowledgeAnalyzerRequest,
            ExecutionJob,
            GetExecuteAnalyzerRequest,
            GetExecuteAnalyzerResponse,
            PluginWorkQueueClient,
        },
    },
    SerDe,
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

impl Workload for GetExecuteAnalyzerResponse {
    fn request_id(&self) -> i64 {
        self.request_id()
    }

    fn maybe_job(self) -> Option<ExecutionJob> {
        self.execution_job()
    }
}

pub struct AnalyzerWorkProcessor {
    analyzer_client: AnalyzerClient,
}

impl AnalyzerWorkProcessor {
    pub async fn new(plugin_id: Uuid) -> Result<Self, Box<dyn std::error::Error>> {
        let upstream_addr_env_var = format!("NOMAD_UPSTREAM_ADDR_plugin-{plugin_id}");
        let address = format!("http://{}", std::env::var(&upstream_addr_env_var)?);
        let client_config: ClientConfiguration = Figment::new()
            .merge(Json::string(&format!("{{\"address\": \"{}\"}}", address)))
            .merge(Env::prefixed("ANALYZER_CLIENT_"))
            .extract()?;

        let analyzer_client = AnalyzerClient::connect(client_config).await?;

        Ok(AnalyzerWorkProcessor { analyzer_client })
    }
}

#[async_trait::async_trait]
impl PluginWorkProcessor for AnalyzerWorkProcessor {
    type Work = GetExecuteAnalyzerResponse;
    type ProducedMessage = ExecutionResult;

    async fn get_work(
        &self,
        plugin_id: Uuid,
        pwq_client: &mut PluginWorkQueueClient,
    ) -> Result<Self::Work, PluginWorkProcessorError> {
        let response = pwq_client
            .get_execute_analyzer(GetExecuteAnalyzerRequest::new(plugin_id))
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
            )
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
            message = "acknowledging analyzer work",
            tenant_id =% tenant_id,
            trace_id =% trace_id,
            event_source_id =% event_source_id,
            plugin_id =% plugin_id,
            request_id =? request_id,
        );

        let execution_hit = process_result.ok();
        let ack_request = AcknowledgeAnalyzerRequest::new(
            request_id,
            execution_hit.is_some(),
            plugin_id,
            tenant_id,
            trace_id,
            event_source_id,
        );

        pwq_client.acknowledge_analyzer(ack_request).await?;

        Ok(())
    }

    async fn process_job(
        &mut self,
        _plugin_id: Uuid,
        job: ExecutionJob,
    ) -> Result<Self::ProducedMessage, PluginWorkProcessorError> {
        let run_analyzer_response = self
            .analyzer_client
            .run_analyzer(RunAnalyzerRequest::new(Update::deserialize(job.data())?))
            .await?;

        Ok(run_analyzer_response.execution_result)
    }
}
