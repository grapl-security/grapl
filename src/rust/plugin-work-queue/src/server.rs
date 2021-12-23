use grapl_utils::future_ext::GraplFutureExt;
use rust_proto::plugin_work_queue::{
    plugin_work_queue_service_server::{
        PluginWorkQueueService,
        PluginWorkQueueServiceServer,
    },
    AcknowledgeAnalyzerRequest,
    AcknowledgeAnalyzerRequestProto,
    AcknowledgeAnalyzerResponse,
    AcknowledgeAnalyzerResponseProto,
    AcknowledgeGeneratorRequest,
    AcknowledgeGeneratorRequestProto,
    AcknowledgeGeneratorResponse,
    AcknowledgeGeneratorResponseProto,
    ExecutionJob,
    GetExecuteAnalyzerRequest,
    GetExecuteAnalyzerRequestProto,
    GetExecuteAnalyzerResponse,
    GetExecuteAnalyzerResponseProto,
    GetExecuteGeneratorRequest,
    GetExecuteGeneratorRequestProto,
    GetExecuteGeneratorResponse,
    GetExecuteGeneratorResponseProto,
    PluginWorkQueueDeserializationError,
    PutExecuteAnalyzerRequest,
    PutExecuteAnalyzerRequestProto,
    PutExecuteAnalyzerResponse,
    PutExecuteAnalyzerResponseProto,
    PutExecuteGeneratorRequest,
    PutExecuteGeneratorRequestProto,
    PutExecuteGeneratorResponse,
    PutExecuteGeneratorResponseProto,
};
use sqlx::{
    Pool,
    Postgres,
};
use tonic::{
    transport::Server,
    Request,
    Response,
    Status,
};

use crate::{
    psql_queue::{
        PsqlQueue,
        PsqlQueueError,
    },
    PluginWorkQueueServiceConfig,
};

#[derive(Debug, thiserror::Error)]
pub enum PluginWorkQueueError {
    #[error("PsqlQueueError")]
    PsqlQueueError(#[from] PsqlQueueError),
    #[error("From<PluginWorkQueueDeserializationError>")]
    DeserializationError(#[from] PluginWorkQueueDeserializationError),
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

pub struct PluginWorkQueue {
    pub(crate) queue: PsqlQueue,
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
    #[allow(dead_code)]
    async fn put_execute_generator(
        &self,
        request: PutExecuteGeneratorRequest,
    ) -> Result<PutExecuteGeneratorResponse, PluginWorkQueueError> {
        let tenant_id = request.execution_job.tenant_id;
        let plugin_id = request.execution_job.plugin_id;
        let data = request.execution_job.data;
        let trace_id = request.trace_id;

        self.queue
            .put_generator_message(plugin_id, data, tenant_id, trace_id)
            .await?;

        Ok(PutExecuteGeneratorResponse {})
    }

    #[allow(dead_code)]
    async fn put_execute_analyzer(
        &self,
        request: PutExecuteAnalyzerRequest,
    ) -> Result<PutExecuteAnalyzerResponse, PluginWorkQueueError> {
        let tenant_id = request.execution_job.tenant_id;
        let plugin_id = request.execution_job.plugin_id;
        let data = request.execution_job.data;
        let trace_id = request.trace_id;

        self.queue
            .put_analyzer_message(plugin_id, data, tenant_id, trace_id)
            .await?;

        Ok(PutExecuteAnalyzerResponse {})
    }

    #[allow(dead_code)]
    async fn get_execute_generator(
        &self,
        _request: GetExecuteGeneratorRequest,
    ) -> Result<GetExecuteGeneratorResponse, PluginWorkQueueError> {
        let message = self.queue.get_analyzer_message().await?;
        let message = match message {
            Some(message) => message,
            None => {
                return Ok(GetExecuteGeneratorResponse {
                    execution_job: None,
                    request_id: 0,
                })
            }
        };
        let execution_job = ExecutionJob {
            tenant_id: message.request.tenant_id,
            plugin_id: message.request.plugin_id,
            data: message.request.pipeline_message,
        };
        Ok(GetExecuteGeneratorResponse {
            execution_job: Some(execution_job),
            request_id: message.request.execution_key.into(),
        })
    }

    #[allow(dead_code)]
    async fn get_execute_analyzer(
        &self,
        _request: GetExecuteAnalyzerRequest,
    ) -> Result<GetExecuteAnalyzerResponse, PluginWorkQueueError> {
        let message = self.queue.get_analyzer_message().await?;
        let message = match message {
            Some(message) => message,
            None => {
                return Ok(GetExecuteAnalyzerResponse {
                    execution_job: None,
                    request_id: 0,
                })
            }
        };
        let execution_job = ExecutionJob {
            tenant_id: message.request.tenant_id,
            plugin_id: message.request.plugin_id,
            data: message.request.pipeline_message,
        };
        Ok(GetExecuteAnalyzerResponse {
            execution_job: Some(execution_job),
            request_id: message.request.execution_key.into(),
        })
    }

    #[allow(dead_code)]
    async fn acknowledge_generator(
        &self,
        request: AcknowledgeGeneratorRequest,
    ) -> Result<AcknowledgeGeneratorResponse, PluginWorkQueueError> {
        self.queue
            .ack_generator_failure(request.request_id.into())
            .await?;
        Ok(AcknowledgeGeneratorResponse {})
    }

    #[allow(dead_code)]
    async fn acknowledge_analyzer(
        &self,
        request: AcknowledgeAnalyzerRequest,
    ) -> Result<AcknowledgeAnalyzerResponse, PluginWorkQueueError> {
        self.queue
            .ack_analyzer_failure(request.request_id.into())
            .await?;
        Ok(AcknowledgeAnalyzerResponse {})
    }
}

#[tonic::async_trait]
impl PluginWorkQueueService for PluginWorkQueue {
    async fn put_execute_generator(
        &self,
        request: Request<PutExecuteGeneratorRequestProto>,
    ) -> Result<Response<PutExecuteGeneratorResponseProto>, Status> {
        let request = request.into_inner();
        let request: PutExecuteGeneratorRequest =
            request.try_into().map_err(PluginWorkQueueError::from)?;
        let response = self.put_execute_generator(request).await?;
        Ok(Response::new(response.into()))
    }

    async fn put_execute_analyzer(
        &self,
        request: Request<PutExecuteAnalyzerRequestProto>,
    ) -> Result<Response<PutExecuteAnalyzerResponseProto>, Status> {
        let request = request.into_inner();
        let request: PutExecuteAnalyzerRequest =
            request.try_into().map_err(PluginWorkQueueError::from)?;
        let response = self.put_execute_analyzer(request).await?;
        Ok(Response::new(response.into()))
    }

    async fn get_execute_generator(
        &self,
        request: Request<GetExecuteGeneratorRequestProto>,
    ) -> Result<Response<GetExecuteGeneratorResponseProto>, Status> {
        let request = request.into_inner();
        let request: GetExecuteGeneratorRequest =
            request.try_into().map_err(PluginWorkQueueError::from)?;
        let response = self.get_execute_generator(request).await?;
        Ok(Response::new(response.into()))
    }

    async fn get_execute_analyzer(
        &self,
        request: Request<GetExecuteAnalyzerRequestProto>,
    ) -> Result<Response<GetExecuteAnalyzerResponseProto>, Status> {
        let request = request.into_inner();
        let request: GetExecuteAnalyzerRequest =
            request.try_into().map_err(PluginWorkQueueError::from)?;
        let response = self.get_execute_analyzer(request).await?;
        Ok(Response::new(response.into()))
    }

    async fn acknowledge_generator(
        &self,
        request: Request<AcknowledgeGeneratorRequestProto>,
    ) -> Result<Response<AcknowledgeGeneratorResponseProto>, Status> {
        let request = request.into_inner();
        let request: AcknowledgeGeneratorRequest =
            request.try_into().map_err(PluginWorkQueueError::from)?;
        let response = self.acknowledge_generator(request).await?;
        Ok(Response::new(response.into()))
    }

    async fn acknowledge_analyzer(
        &self,
        request: Request<AcknowledgeAnalyzerRequestProto>,
    ) -> Result<Response<AcknowledgeAnalyzerResponseProto>, Status> {
        let request = request.into_inner();
        let request: AcknowledgeAnalyzerRequest =
            request.try_into().map_err(PluginWorkQueueError::from)?;
        let response = self.acknowledge_analyzer(request).await?;
        Ok(Response::new(response.into()))
    }
}

pub async fn exec_service(
    service_config: PluginWorkQueueServiceConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter
        .set_serving::<PluginWorkQueueServiceServer<PluginWorkQueue>>()
        .await;

    tracing::info!(
        message="Connecting to plugin registry table",
        service_config=?service_config,
    );
    let postgres_address = format!(
        "postgresql://{}:{}@{}:{}",
        service_config.plugin_work_queue_db_username,
        service_config.plugin_work_queue_db_password,
        service_config.plugin_work_queue_db_hostname,
        service_config.plugin_work_queue_db_port,
    );

    let plugin_work_queue: PluginWorkQueue = PluginWorkQueue::from(
        sqlx::PgPool::connect(&postgres_address)
            .timeout(std::time::Duration::from_secs(5))
            .await??,
    );

    sqlx::migrate!().run(&plugin_work_queue.queue.pool).await?;

    Server::builder()
        .trace_fn(|request| {
            tracing::info_span!(
                "PluginWorkQueue",
                headers = ?request.headers(),
                method = ?request.method(),
                uri = %request.uri(),
                extensions = ?request.extensions(),
            )
        })
        .add_service(health_service)
        .add_service(PluginWorkQueueServiceServer::new(plugin_work_queue))
        .serve(service_config.plugin_work_queue_bind_address)
        .await?;

    Ok(())
}
