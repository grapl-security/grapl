use std::time::Duration;

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
        },
        pipeline::v1beta1::Envelope,
    },
    protocol::{
        healthcheck::HealthcheckStatus,
        status::Status,
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
    #[error("Timeout {0}")]
    Timeout(#[from] tokio::time::error::Elapsed),
    #[error("Sqlx {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("Kafka {0}")]
    Kafka(#[from] kafka::ConfigurationError),
}

impl From<PluginWorkQueueError> for Status {
    fn from(err: PluginWorkQueueError) -> Self {
        match err {
            PluginWorkQueueError::PsqlQueueError(_) => Status::internal("Sql Error"),
            PluginWorkQueueError::KafkaProducerError(_) => Status::internal("Kafka Produce error"),
            PluginWorkQueueError::DeserializationError(_) => {
                Status::invalid_argument("Invalid argument")
            }
        }
    }
}

#[derive(Clone)]
pub struct PluginWorkQueue {
    queue: PsqlQueue,
    generator_producer: Producer<Envelope<GraphDescription>>,
}

impl PluginWorkQueue {
    pub async fn try_from(configs: &ConfigUnion) -> Result<Self, PluginWorkQueueInitError> {
        let psql_queue = PsqlQueue::try_from(configs.db_config.clone()).await?;
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
        let data = execution_job.data().clone();

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
        let data = execution_job.data().clone();

        self.queue
            .put_analyzer_message(plugin_id, tenant_id, trace_id, event_source_id, data)
            .await?;

        Ok(v1beta1::PushExecuteAnalyzerResponse {})
    }

    #[tracing::instrument(skip(self, _request), err)]
    async fn get_execute_generator(
        &self,
        _request: v1beta1::GetExecuteGeneratorRequest,
    ) -> Result<v1beta1::GetExecuteGeneratorResponse, PluginWorkQueueError> {
        let message = self.queue.get_generator_message().await?;
        let message = match message {
            Some(message) => message,
            None => {
                return Ok(v1beta1::GetExecuteGeneratorResponse {
                    execution_job: None,
                    request_id: 0,
                })
            }
        };
        let execution_job = v1beta1::ExecutionJob::new(
            message.request.pipeline_message.into(),
            message.request.tenant_id,
            message.request.trace_id,
            message.request.event_source_id,
        );
        Ok(v1beta1::GetExecuteGeneratorResponse {
            execution_job: Some(execution_job),
            request_id: message.request.execution_key.into(),
        })
    }

    #[tracing::instrument(skip(self, _request), err)]
    async fn get_execute_analyzer(
        &self,
        _request: v1beta1::GetExecuteAnalyzerRequest,
    ) -> Result<v1beta1::GetExecuteAnalyzerResponse, PluginWorkQueueError> {
        let message = self.queue.get_analyzer_message().await?;
        let message = match message {
            Some(message) => message,
            None => {
                return Ok(v1beta1::GetExecuteAnalyzerResponse {
                    execution_job: None,
                    request_id: 0,
                })
            }
        };
        let execution_job = v1beta1::ExecutionJob::new(
            message.request.pipeline_message.into(),
            message.request.tenant_id,
            message.request.trace_id,
            message.request.event_source_id,
        );
        Ok(v1beta1::GetExecuteAnalyzerResponse {
            execution_job: Some(execution_job),
            request_id: message.request.execution_key.into(),
        })
    }

    #[tracing::instrument(skip(self, request), err)]
    async fn acknowledge_generator(
        &self,
        request: v1beta1::AcknowledgeGeneratorRequest,
    ) -> Result<v1beta1::AcknowledgeGeneratorResponse, PluginWorkQueueError> {
        let status = match request.graph_description() {
            Some(graph_description) => {
                self.generator_producer
                    .send(Envelope::new(
                        request.tenant_id(),
                        request.trace_id(),
                        request.event_source_id(),
                        graph_description.clone(),
                    ))
                    .await?;
                psql_queue::Status::Processed
            }
            None => psql_queue::Status::Failed,
        };
        self.queue
            .ack_generator(request.request_id().into(), status)
            .await?;
        Ok(v1beta1::AcknowledgeGeneratorResponse {})
    }

    #[tracing::instrument(skip(self, request), err)]
    async fn acknowledge_analyzer(
        &self,
        request: v1beta1::AcknowledgeAnalyzerRequest,
    ) -> Result<v1beta1::AcknowledgeAnalyzerResponse, PluginWorkQueueError> {
        let status = match request.success {
            true => psql_queue::Status::Processed,
            false => psql_queue::Status::Failed,
        };
        self.queue
            .ack_analyzer(request.request_id.into(), status)
            .await?;
        Ok(v1beta1::AcknowledgeAnalyzerResponse {})
    }
}

pub async fn exec_service(configs: ConfigUnion) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!(
        message="Connecting to plugin-work-queue table",
        db_config=?configs.db_config,
    );

    let plugin_work_queue = PluginWorkQueue::try_from(&configs).await?;

    tracing::info!(message = "Performing migration",);
    sqlx::migrate!().run(&plugin_work_queue.queue.pool).await?;

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
