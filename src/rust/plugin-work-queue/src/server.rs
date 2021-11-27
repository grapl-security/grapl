use tonic::{Request, Response, Status};
use tonic::transport::Server;
use rust_proto::plugin_work_queue::plugin_work_queue_service_server::{PluginWorkQueueService, PluginWorkQueueServiceServer};
use rust_proto::plugin_work_queue::{AcknowledgeRequest, AcknowledgeResponse, GetExecuteAnalyzerRequest, GetExecuteAnalyzerResponse, GetExecuteGeneratorRequest, GetExecuteGeneratorResponse, PutExecuteAnalyzerRequest, PutExecuteAnalyzerResponse, PutExecuteGeneratorRequest, PutExecuteGeneratorResponse};
use rust_proto::plugin_work_queue::{
    _AcknowledgeRequest,
    _AcknowledgeResponse,
    _GetExecuteAnalyzerRequest,
    _GetExecuteAnalyzerResponse,
    _GetExecuteGeneratorRequest,
    _GetExecuteGeneratorResponse,
    _PutExecuteAnalyzerRequest,
    _PutExecuteAnalyzerResponse,
    _PutExecuteGeneratorRequest,
    _PutExecuteGeneratorResponse,
};

#[derive(Debug, thiserror::Error)]
pub enum PluginWorkQueueError {

}

pub struct PluginWorkQueue {}

impl PluginWorkQueue {
    async fn _put_execute_generator(&self, _request: PutExecuteGeneratorRequest) -> Result<PutExecuteGeneratorResponse, PluginWorkQueueError> {
        todo!()
    }

    async fn _put_execute_analyzer(&self, _request: PutExecuteAnalyzerRequest) -> Result<PutExecuteAnalyzerResponse, PluginWorkQueueError> {
        todo!()
    }

    async fn _get_execute_generator(&self, _request: GetExecuteGeneratorRequest) -> Result<GetExecuteGeneratorResponse, PluginWorkQueueError> {
        todo!()
    }

    async fn _get_execute_analyzer(&self, _request: GetExecuteAnalyzerRequest) -> Result<GetExecuteAnalyzerResponse, PluginWorkQueueError> {
        todo!()
    }

    async fn _acknowledge(&self, _request: AcknowledgeRequest) -> Result<AcknowledgeResponse, PluginWorkQueueError> {
        todo!()
    }
}

#[tonic::async_trait]
impl PluginWorkQueueService for PluginWorkQueue {
    async fn put_execute_generator(&self, _request: Request<_PutExecuteGeneratorRequest>) -> Result<Response<_PutExecuteGeneratorResponse>, Status> {
        todo!()
    }

    async fn put_execute_analyzer(&self, _request: Request<_PutExecuteAnalyzerRequest>) -> Result<Response<_PutExecuteAnalyzerResponse>, Status> {
        todo!()
    }

    async fn get_execute_generator(&self, _request: Request<_GetExecuteGeneratorRequest>) -> Result<Response<_GetExecuteGeneratorResponse>, Status> {
        todo!()
    }

    async fn get_execute_analyzer(&self, _request: Request<_GetExecuteAnalyzerRequest>) -> Result<Response<_GetExecuteAnalyzerResponse>, Status> {
        todo!()
    }

    async fn acknowledge(&self, _request: Request<_AcknowledgeRequest>) -> Result<Response<_AcknowledgeResponse>, Status> {
        todo!()
    }
}



pub async fn exec_service()  -> Result<(), Box<dyn std::error::Error>> {
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
