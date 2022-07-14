use std::time::SystemTime;

use kafka::{
    config::ProducerConfig,
    Producer,
};
use plugin_work_queue::client::PluginWorkQueueServiceClient;
use rust_proto::graplinc::grapl::{
    api::{
        graph::v1beta1::GraphDescription,
        plugin_sdk::generators::v1beta1::{
            client::GeneratorServiceClient,
            RunGeneratorRequest,
        },
        plugin_work_queue::v1beta1::{
            AcknowledgeGeneratorRequest,
            ExecutionJob,
            GetExecuteGeneratorRequest,
            GetExecuteGeneratorResponse,
            PluginWorkQueueServiceClientError,
        },
    },
    pipeline::{
        v1beta1::Metadata,
        v1beta2::Envelope,
    },
};

use super::{
    plugin_work_processor::{
        RequestId,
        Workload,
    },
    PluginWorkProcessor,
};
use crate::{
    config::PluginExecutorConfig,
    sidecar_client::generator_client::get_generator_client,
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
    kafka_producer: Producer<Envelope<GraphDescription>>,
}

impl GeneratorWorkProcessor {
    pub async fn new(
        config: &PluginExecutorConfig,
        producer_config: ProducerConfig,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let generator_service_client = get_generator_client(config.plugin_id).await?;
        let kafka_producer = Producer::new(producer_config)?;
        Ok(GeneratorWorkProcessor {
            generator_service_client,
            kafka_producer,
        })
    }
}

#[async_trait::async_trait]
impl PluginWorkProcessor for GeneratorWorkProcessor {
    type Work = GetExecuteGeneratorResponse;

    async fn get_work(
        &self,
        config: &PluginExecutorConfig,
        pwq_client: &mut PluginWorkQueueServiceClient,
    ) -> Result<Self::Work, PluginWorkQueueServiceClientError> {
        let plugin_id = config.plugin_id;
        let get_request = GetExecuteGeneratorRequest { plugin_id };
        pwq_client.get_execute_generator(get_request).await
    }

    async fn ack_work(
        &self,
        config: &PluginExecutorConfig,
        pwq_client: &mut PluginWorkQueueServiceClient,
        request_id: RequestId,
        success: bool,
    ) -> Result<(), PluginWorkQueueServiceClientError> {
        let plugin_id = config.plugin_id;
        let ack_request = AcknowledgeGeneratorRequest {
            plugin_id,
            request_id,
            success,
        };
        pwq_client
            .acknowledge_generator(ack_request)
            .await
            .map(|_| ())
    }

    async fn process_job(
        &mut self,
        config: &PluginExecutorConfig,
        job: ExecutionJob,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let run_generator_response = self
            .generator_service_client
            .run_generator(RunGeneratorRequest { data: job.data })
            .await?;

        let kafka_msg = {
            let tenant_id = config.tenant_id;

            let trace_id = uuid::Uuid::new_v4(); // FIXME // TODO
            let retry_count = 0;
            let created_time = SystemTime::now();
            let last_updated_time = created_time;
            let event_source_id = uuid::Uuid::new_v4(); // FIXME // TODO
            Envelope::new(
                Metadata::new(
                    tenant_id,
                    trace_id,
                    retry_count,
                    created_time,
                    last_updated_time,
                    event_source_id,
                ),
                run_generator_response.generated_graph.graph_description,
            )
        };

        self.kafka_producer.send(kafka_msg).await?;
        Ok(()) // TODO replace with above
    }
}
