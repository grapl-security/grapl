use rust_proto::{
    client_factory::{
        build_grpc_client,
        services::AnalyzerClientConfig,
    },
    graplinc::grapl::api::{
        plugin_sdk::analyzers::v1beta1::{
            client::AnalyzerServiceClient,
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
            PluginWorkQueueServiceClient,
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
use crate::config::PluginExecutorConfig;

impl Workload for GetExecuteAnalyzerResponse {
    fn request_id(&self) -> i64 {
        self.request_id()
    }

    fn maybe_job(self) -> Option<ExecutionJob> {
        self.execution_job()
    }
}

pub struct AnalyzerWorkProcessor {
    analyzer_service_client: AnalyzerServiceClient,
}

impl AnalyzerWorkProcessor {
    pub async fn new(config: &PluginExecutorConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let plugin_id = config.plugin_id;
        let upstream_addr_env_var = format!("NOMAD_UPSTREAM_ADDR_plugin-{plugin_id}");
        let upstream_addr = std::env::var(&upstream_addr_env_var).expect(&upstream_addr_env_var);
        let address = format!("http://{upstream_addr}");

        tracing::info!(message = "connecting to analyzer plugin", address = address);

        let client_config = AnalyzerClientConfig {
            analyzer_client_address: address.parse().expect("analyzer client address"),
        };

        let analyzer_service_client = build_grpc_client(client_config).await?;

        Ok(AnalyzerWorkProcessor {
            analyzer_service_client,
        })
    }
}

#[async_trait::async_trait]
impl PluginWorkProcessor for AnalyzerWorkProcessor {
    type Work = GetExecuteAnalyzerResponse;
    type ProducedMessage = ExecutionResult;

    async fn get_work(
        &self,
        config: &PluginExecutorConfig,
        pwq_client: &mut PluginWorkQueueServiceClient,
    ) -> Result<Self::Work, PluginWorkProcessorError> {
        let plugin_id = config.plugin_id;
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
        config: &PluginExecutorConfig,
        pwq_client: &mut PluginWorkQueueServiceClient,
        process_result: Result<Self::ProducedMessage, PluginWorkProcessorError>,
        request_id: RequestId,
        tenant_id: Uuid,
        trace_id: Uuid,
        event_source_id: Uuid,
    ) -> Result<(), PluginWorkProcessorError> {
        let plugin_id = config.plugin_id;

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
        _config: &PluginExecutorConfig,
        job: ExecutionJob,
    ) -> Result<Self::ProducedMessage, PluginWorkProcessorError> {
        let run_analyzer_response = self
            .analyzer_service_client
            .run_analyzer(RunAnalyzerRequest::new(Update::deserialize(job.data())?))
            .await?;

        Ok(run_analyzer_response.execution_result)
    }
}
