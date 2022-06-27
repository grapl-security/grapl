use std::time::Duration;

use rust_proto::{
    graplinc::grapl::api::plugin_work_queue::{
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
    SerDeError,
};
use sqlx::{
    Pool,
    Postgres,
};
use tokio::net::TcpListener;

use crate::{
    psql_queue::{
        self,
        PsqlQueue,
        PsqlQueueError,
    },
    PluginWorkQueueServiceConfig,
};

#[derive(Debug, thiserror::Error)]
pub enum PluginWorkQueueError {
    #[error("PsqlQueueError {0}")]
    PsqlQueueError(#[from] PsqlQueueError),
    #[error("PluginWorkQueueDeserializationError {0}")]
    DeserializationError(#[from] SerDeError),
}

#[derive(Debug, thiserror::Error)]
pub enum PluginWorkQueueInitError {
    #[error("Timeout {0}")]
    Timeout(#[from] tokio::time::error::Elapsed),
    #[error("Sqlx {0}")]
    Sqlx(#[from] sqlx::Error),
}

impl From<PluginWorkQueueError> for Status {
    fn from(err: PluginWorkQueueError) -> Self {
        match err {
            PluginWorkQueueError::PsqlQueueError(_) => Status::internal("Sql Error"),
            PluginWorkQueueError::DeserializationError(_) => {
                Status::invalid_argument("Invalid argument")
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct PluginWorkQueue {
    queue: PsqlQueue,
}

impl From<PsqlQueue> for PluginWorkQueue {
    fn from(queue: PsqlQueue) -> Self {
        Self { queue }
    }
}

impl From<Pool<Postgres>> for PluginWorkQueue {
    fn from(pool: Pool<Postgres>) -> Self {
        Self {
            queue: PsqlQueue { pool },
        }
    }
}

impl PluginWorkQueue {
    pub async fn try_from(
        service_config: &PluginWorkQueueServiceConfig,
    ) -> Result<Self, PluginWorkQueueInitError> {
        Ok(Self::from(
            PsqlQueue::try_from(service_config.db_config.clone()).await?,
        ))
    }
}
#[async_trait::async_trait]
impl PluginWorkQueueApi for PluginWorkQueue {
    type Error = PluginWorkQueueError;

    #[tracing::instrument(skip(self, request), err)]
    async fn put_execute_generator(
        &self,
        request: v1beta1::PutExecuteGeneratorRequest,
    ) -> Result<v1beta1::PutExecuteGeneratorResponse, PluginWorkQueueError> {
        let tenant_id = request.execution_job.tenant_id;
        let plugin_id = request.execution_job.plugin_id;
        let data = request.execution_job.data;

        self.queue
            .put_generator_message(plugin_id, data, tenant_id)
            .await?;

        Ok(v1beta1::PutExecuteGeneratorResponse {})
    }

    #[tracing::instrument(skip(self, request), err)]
    async fn put_execute_analyzer(
        &self,
        request: v1beta1::PutExecuteAnalyzerRequest,
    ) -> Result<v1beta1::PutExecuteAnalyzerResponse, PluginWorkQueueError> {
        let tenant_id = request.execution_job.tenant_id;
        let plugin_id = request.execution_job.plugin_id;
        let data = request.execution_job.data;

        self.queue
            .put_analyzer_message(plugin_id, data, tenant_id)
            .await?;

        Ok(v1beta1::PutExecuteAnalyzerResponse {})
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
        let execution_job = v1beta1::ExecutionJob {
            tenant_id: message.request.tenant_id,
            plugin_id: message.request.plugin_id,
            data: message.request.pipeline_message,
        };
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
        let execution_job = v1beta1::ExecutionJob {
            tenant_id: message.request.tenant_id,
            plugin_id: message.request.plugin_id,
            data: message.request.pipeline_message,
        };
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
        let status = match request.success {
            true => psql_queue::Status::Processed,
            false => psql_queue::Status::Failed,
        };
        self.queue
            .ack_generator(request.request_id.into(), status)
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

pub async fn exec_service(
    service_config: PluginWorkQueueServiceConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!(
        message="Connecting to plugin-work-queue table",
        service_config=?service_config,
    );

    let plugin_work_queue = PluginWorkQueue::try_from(&service_config).await?;

    tracing::info!(message = "Performing migration",);
    sqlx::migrate!().run(&plugin_work_queue.queue.pool).await?;

    tracing::info!(message = "Binding service",);
    let addr = service_config.plugin_work_queue_bind_address;
    let healthcheck_polling_interval_ms =
        service_config.plugin_work_queue_healthcheck_polling_interval_ms;

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

    server.serve().await
}
