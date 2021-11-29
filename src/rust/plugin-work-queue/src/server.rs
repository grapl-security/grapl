use rust_proto::plugin_work_queue::{
    plugin_work_queue_service_server::{
        PluginWorkQueueService,
        PluginWorkQueueServiceServer,
    },
    AcknowledgeRequest,
    AcknowledgeResponse,
    GetExecuteAnalyzerRequest,
    GetExecuteAnalyzerResponse,
    GetExecuteGeneratorRequest,
    GetExecuteGeneratorResponse,
    PutExecuteAnalyzerRequest,
    PutExecuteAnalyzerResponse,
    PutExecuteGeneratorRequest,
    PutExecuteGeneratorResponse,
    AcknowledgeRequestProto,
    AcknowledgeResponseProto,
    GetExecuteAnalyzerRequestProto,
    GetExecuteAnalyzerResponseProto,
    GetExecuteGeneratorRequestProto,
    GetExecuteGeneratorResponseProto,
    PutExecuteAnalyzerRequestProto,
    PutExecuteAnalyzerResponseProto,
    PutExecuteGeneratorRequestProto,
    PutExecuteGeneratorResponseProto,
};
use tonic::{
    transport::Server,
    Request,
    Response,
    Status,
};

#[derive(Debug, thiserror::Error)]
pub enum PluginWorkQueueError {}

pub struct PluginWorkQueue {}

impl PluginWorkQueue {
    #[allow(dead_code)]
    async fn put_execute_generator(
        &self,
        _request: PutExecuteGeneratorRequest,
    ) -> Result<PutExecuteGeneratorResponse, PluginWorkQueueError> {
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

pub async fn exec_service() -> Result<(), Box<dyn std::error::Error>> {
    let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter
        .set_serving::<PluginWorkQueueServiceServer<PluginWorkQueue>>()
        .await;

    let addr = "[::1]:50051".parse().unwrap();
    let plugin_work_queue = PluginWorkQueue {};

    tracing::info!(
        message="HealthServer + PluginWorkQueue listening",
        addr=?addr,
    );

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
        .serve(addr)
        .await?;

    Ok(())
}
