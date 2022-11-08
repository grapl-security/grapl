use std::time::Duration;

use grapl_config::PostgresClient;
use kafka::{
    Producer,
    ProducerError,
};
use rust_proto::{
    graplinc::grapl::{
        api::{
            graph::v1beta1::GraphDescription,
            plugin_work_queue::{
                v1beta1,
                v1beta1::{
                    PluginWorkQueueApi,
                    PluginWorkQueueServer,
                },
            },
            protocol::{
                healthcheck::HealthcheckStatus,
                status::Status,
            },
        },
        pipeline::v1beta1::Envelope,
    },
    SerDeError,
};
use tokio::net::TcpListener;

use crate::{
    psql_queue::{
        self,
        PsqlQueue,
        PsqlQueueError,
    },
    ConfigUnion,
};

#[derive(Debug, thiserror::Error)]
pub enum PluginWorkQueueError {
    #[error("PsqlQueueError {0}")]
    PsqlQueueError(#[from] PsqlQueueError),
    #[error("PluginWorkQueueDeserializationError {0}")]
    DeserializationError(#[from] SerDeError),
    #[error("KafkaProducerError {0}")]
    KafkaProducerError(#[from] ProducerError),
}

#[derive(Debug, thiserror::Error)]
pub enum PluginWorkQueueInitError {
    #[error(transparent)]
    DbInit(#[from] grapl_config::PostgresDbInitError),
    #[error("Kafka {0}")]
    Kafka(#[from] kafka::ConfigurationError),
}

impl From<PluginWorkQueueError> for Status {
    fn from(err: PluginWorkQueueError) -> Self {
        match err {
            PluginWorkQueueError::PsqlQueueError(_) => Status::unknown("Sql Error"),
            PluginWorkQueueError::KafkaProducerError(_) => Status::unknown("Kafka Produce error"),
            PluginWorkQueueError::DeserializationError(_) => {
                Status::invalid_argument("Invalid argument")
            }
        }
    }
}

#[derive(Clone)]
pub struct PluginWorkQueue {
    queue: PsqlQueue,
    generator_producer: Producer<GraphDescription>,
}

impl PluginWorkQueue {
    pub async fn try_from(configs: &ConfigUnion) -> Result<Self, PluginWorkQueueInitError> {
        let psql_queue = PsqlQueue::init_with_config(configs.db_config.clone()).await?;
        let generator_producer = Producer::new(configs.generator_producer_config.clone())?;
        Ok(Self {
            queue: psql_queue,
            generator_producer,
        })
    }
}
#[async_trait::async_trait]
impl PluginWorkQueueApi for PluginWorkQueue {
    type Error = PluginWorkQueueError;

    #[tracing::instrument(skip(self, request), err)]
    async fn push_execute_generator(
        &self,
        request: v1beta1::PushExecuteGeneratorRequest,
    ) -> Result<v1beta1::PushExecuteGeneratorResponse, PluginWorkQueueError> {
        let plugin_id = request.plugin_id();
        let execution_job = request.execution_job();
        let tenant_id = execution_job.tenant_id();
        let trace_id = execution_job.trace_id();
        let event_source_id = execution_job.event_source_id();
        let data = execution_job.data();

        tracing::debug!(
            message = "enqueueing generator execution",
            tenant_id =% tenant_id,
            trace_id =% trace_id,
            event_source_id =% event_source_id,
            plugin_id =% plugin_id,
        );

        self.queue
            .put_generator_message(plugin_id, tenant_id, trace_id, event_source_id, data)
            .await?;

        Ok(v1beta1::PushExecuteGeneratorResponse {})
    }

    #[tracing::instrument(skip(self, request), err)]
    async fn push_execute_analyzer(
        &self,
        request: v1beta1::PushExecuteAnalyzerRequest,
    ) -> Result<v1beta1::PushExecuteAnalyzerResponse, PluginWorkQueueError> {
        let plugin_id = request.plugin_id();
        let execution_job = request.execution_job();
        let tenant_id = execution_job.tenant_id();
        let trace_id = execution_job.trace_id();
        let event_source_id = execution_job.event_source_id();
        let data = execution_job.data();

        tracing::debug!(
            message = "enqueueing analyzer execution",
            tenant_id =% tenant_id,
            trace_id =% trace_id,
            event_source_id =% event_source_id,
            plugin_id =% plugin_id,
        );

        self.queue
            .put_analyzer_message(plugin_id, tenant_id, trace_id, event_source_id, data)
            .await?;

        Ok(v1beta1::PushExecuteAnalyzerResponse {})
    }

    #[tracing::instrument(skip(self, request), err)]
    async fn get_execute_generator(
        &self,
        request: v1beta1::GetExecuteGeneratorRequest,
    ) -> Result<v1beta1::GetExecuteGeneratorResponse, PluginWorkQueueError> {
        let plugin_id = request.plugin_id();
        let message = self.queue.get_generator_message(plugin_id).await?;
        let message = match message {
            Some(message) => message,
            None => {
                tracing::warn!(
                    message = "found no generator executions",
                    plugin_id =% plugin_id,
                );
                return Ok(v1beta1::GetExecuteGeneratorResponse::new(None, 0));
            }
        };

        let tenant_id = message.request.tenant_id;
        let trace_id = message.request.trace_id;
        let event_source_id = message.request.event_source_id;

        tracing::debug!(
            message = "retrieving generator execution",
            tenant_id =% tenant_id,
            trace_id =% trace_id,
            event_source_id =% event_source_id,
            plugin_id =% plugin_id,
        );

        let execution_job = v1beta1::ExecutionJob::new(
            message.request.pipeline_message.into(),
            tenant_id,
            trace_id,
            event_source_id,
        );
        Ok(v1beta1::GetExecuteGeneratorResponse::new(
            Some(execution_job),
            message.request.execution_key.into(),
        ))
    }

    #[tracing::instrument(skip(self, request), err)]
    async fn get_execute_analyzer(
        &self,
        request: v1beta1::GetExecuteAnalyzerRequest,
    ) -> Result<v1beta1::GetExecuteAnalyzerResponse, PluginWorkQueueError> {
        let message = self.queue.get_analyzer_message(request.plugin_id()).await?;
        let message = match message {
            Some(message) => message,
            None => {
                tracing::warn!(
                    message = "found no analyzer executions",
                    plugin_id =% request.plugin_id(),
                );
                return Ok(v1beta1::GetExecuteAnalyzerResponse::new(None, 0));
            }
        };

        let tenant_id = message.request.tenant_id;
        let trace_id = message.request.trace_id;
        let event_source_id = message.request.event_source_id;

        tracing::debug!(
            message = "retrieving analyzer execution",
            tenant_id =% tenant_id,
            trace_id =% trace_id,
            event_source_id =% event_source_id,
            plugin_id =% request.plugin_id(),
        );

        let execution_job = v1beta1::ExecutionJob::new(
            message.request.pipeline_message.into(),
            tenant_id,
            trace_id,
            event_source_id,
        );
        Ok(v1beta1::GetExecuteAnalyzerResponse::new(
            Some(execution_job),
            message.request.execution_key.into(),
        ))
    }

    #[tracing::instrument(skip(self, request), err)]
    async fn acknowledge_generator(
        &self,
        request: v1beta1::AcknowledgeGeneratorRequest,
    ) -> Result<v1beta1::AcknowledgeGeneratorResponse, PluginWorkQueueError> {
        let tenant_id = request.tenant_id();
        let trace_id = request.trace_id();
        let event_source_id = request.event_source_id();
        let request_id = request.request_id();
        let plugin_id = request.plugin_id();

        let status = match request.graph_description() {
            Some(graph_description) => {
                tracing::debug!(
                    message = "publishing generator execution result",
                    tenant_id =% tenant_id,
                    trace_id =% trace_id,
                    event_source_id =% event_source_id,
                    plugin_id =% plugin_id,
                );

                self.generator_producer
                    .send(Envelope::new(
                        tenant_id,
                        trace_id,
                        event_source_id,
                        graph_description,
                    ))
                    .await?;

                psql_queue::Status::Processed
            }
            None => psql_queue::Status::Failed,
        };

        tracing::debug!(
            message = "acknowledging generator execution",
            tenant_id =% tenant_id,
            trace_id =% trace_id,
            event_source_id =% event_source_id,
            plugin_id =% plugin_id,
            status =? status,
        );

        self.queue.ack_generator(request_id.into(), status).await?;

        Ok(v1beta1::AcknowledgeGeneratorResponse {})
    }

    #[tracing::instrument(skip(self, request), err)]
    async fn acknowledge_analyzer(
        &self,
        request: v1beta1::AcknowledgeAnalyzerRequest,
    ) -> Result<v1beta1::AcknowledgeAnalyzerResponse, PluginWorkQueueError> {
        let tenant_id = request.tenant_id();
        let trace_id = request.trace_id();
        let event_source_id = request.event_source_id();
        let plugin_id = request.plugin_id();

        let status = match request.success() {
            true => psql_queue::Status::Processed,
            false => psql_queue::Status::Failed,
        };

        tracing::debug!(
            message = "acknowledging analyzer execution",
            tenant_id =% tenant_id,
            trace_id =% trace_id,
            event_source_id =% event_source_id,
            plugin_id =% plugin_id,
            status =? status,
        );

        self.queue
            .ack_analyzer(request.request_id().into(), status)
            .await?;
        Ok(v1beta1::AcknowledgeAnalyzerResponse {})
    }

    #[tracing::instrument(skip(self, _request), err)]
    async fn queue_depth_for_generator(
        &self,
        _request: v1beta1::QueueDepthForGeneratorRequest,
    ) -> Result<v1beta1::QueueDepthForGeneratorResponse, PluginWorkQueueError> {
        todo!()
    }

    #[tracing::instrument(skip(self, _request), err)]
    async fn queue_depth_for_analyzer(
        &self,
        _request: v1beta1::QueueDepthForAnalyzerRequest,
    ) -> Result<v1beta1::QueueDepthForAnalyzerResponse, PluginWorkQueueError> {
        todo!()
    }
}

pub async fn exec_service(configs: ConfigUnion) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!(
        message="Connecting to plugin-work-queue table",
        db_config=?configs.db_config,
    );

    let plugin_work_queue = PluginWorkQueue::try_from(&configs).await?;

    tracing::info!(message = "Binding service",);
    let addr = configs.service_config.plugin_work_queue_bind_address;
    let healthcheck_polling_interval_ms = configs
        .service_config
        .plugin_work_queue_healthcheck_polling_interval_ms;

    let (server, _shutdown_tx) = PluginWorkQueueServer::new(
        plugin_work_queue,
        TcpListener::bind(addr.clone()).await?,
        || async { Ok(HealthcheckStatus::Serving) }, // FIXME: this is garbage
        Duration::from_millis(healthcheck_polling_interval_ms),
    );

    tracing::info!(
        message = "starting gRPC server",
        socket_address = %addr,
    );

    Ok(server.serve().await?)
}
