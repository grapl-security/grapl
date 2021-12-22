use grapl_utils::future_ext::GraplFutureExt;
use rust_proto::plugin_work_queue::{
    plugin_work_queue_service_server::{
        PluginWorkQueueService,
        PluginWorkQueueServiceServer,
    },
    AcknowledgeRequest,
    AcknowledgeRequestProto,
    AcknowledgeResponse,
    AcknowledgeResponseProto,
    GetExecuteAnalyzerRequest,
    GetExecuteAnalyzerRequestProto,
    GetExecuteAnalyzerResponse,
    GetExecuteAnalyzerResponseProto,
    GetExecuteGeneratorRequest,
    GetExecuteGeneratorRequestProto,
    GetExecuteGeneratorResponse,
    GetExecuteGeneratorResponseProto,
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
    psql_queue::PsqlQueue,
    PluginWorkQueueServiceConfig,
};

#[derive(Debug, thiserror::Error)]
pub enum PluginWorkQueueError {}

pub struct PluginWorkQueue {
    pub(crate) pool: PsqlQueue,
}

impl From<PsqlQueue> for PluginWorkQueue {
    fn from(pool: PsqlQueue) -> Self {
        Self { pool }
    }
}

impl From<Pool<Postgres>> for PluginWorkQueue {
    fn from(pool: Pool<Postgres>) -> Self {
        Self {
            pool: PsqlQueue::new(pool),
        }
    }
}

impl PluginWorkQueue {
    #[allow(dead_code)]
    async fn put_execute_generator(
        &self,
        _request: PutExecuteGeneratorRequest,
    ) -> Result<PutExecuteGeneratorResponse, PluginWorkQueueError> {
        // sqlx::query!(
        //     "SELECT execution_key from plugin_executions LIMIT 1;"
        // )
        //     .fetch_one(&self.pool);
        todo!()
    }

    #[allow(dead_code)]
    async fn put_execute_analyzer(
        &self,
        _request: PutExecuteAnalyzerRequest,
    ) -> Result<PutExecuteAnalyzerResponse, PluginWorkQueueError> {
        todo!()
    }

    #[allow(dead_code)]
    async fn get_execute_generator(
        &self,
        _request: GetExecuteGeneratorRequest,
    ) -> Result<GetExecuteGeneratorResponse, PluginWorkQueueError> {
        todo!()
    }

    #[allow(dead_code)]
    async fn get_execute_analyzer(
        &self,
        _request: GetExecuteAnalyzerRequest,
    ) -> Result<GetExecuteAnalyzerResponse, PluginWorkQueueError> {
        todo!()
    }

    #[allow(dead_code)]
    async fn acknowledge(
        &self,
        _request: AcknowledgeRequest,
    ) -> Result<AcknowledgeResponse, PluginWorkQueueError> {
        todo!()
    }
}

#[tonic::async_trait]
impl PluginWorkQueueService for PluginWorkQueue {
    async fn put_execute_generator(
        &self,
        _request: Request<PutExecuteGeneratorRequestProto>,
    ) -> Result<Response<PutExecuteGeneratorResponseProto>, Status> {
        todo!()
    }

    async fn put_execute_analyzer(
        &self,
        _request: Request<PutExecuteAnalyzerRequestProto>,
    ) -> Result<Response<PutExecuteAnalyzerResponseProto>, Status> {
        todo!()
    }

    async fn get_execute_generator(
        &self,
        _request: Request<GetExecuteGeneratorRequestProto>,
    ) -> Result<Response<GetExecuteGeneratorResponseProto>, Status> {
        todo!()
    }

    async fn get_execute_analyzer(
        &self,
        _request: Request<GetExecuteAnalyzerRequestProto>,
    ) -> Result<Response<GetExecuteAnalyzerResponseProto>, Status> {
        todo!()
    }

    async fn acknowledge(
        &self,
        _request: Request<AcknowledgeRequestProto>,
    ) -> Result<Response<AcknowledgeResponseProto>, Status> {
        todo!()
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

    sqlx::migrate!().run(&plugin_work_queue.pool.pool).await?;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn migrate_for_tests() -> Result<(), Box<dyn std::error::Error>> {
        let postgres_address = "postgresql://postgres:postgres@localhost:5432";

        let plugin_work_queue: PluginWorkQueue = PluginWorkQueue::from(
            sqlx::PgPool::connect(&postgres_address)
                .timeout(std::time::Duration::from_secs(5))
                .await??,
        );

        sqlx::migrate!().run(&plugin_work_queue.pool.pool).await?;
        Ok(())
    }
}
