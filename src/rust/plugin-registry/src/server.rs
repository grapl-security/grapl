use std::net::SocketAddr;

use rust_proto::plugin_registry::{
    plugin_registry_service_server::{
        PluginRegistryService,
        PluginRegistryServiceServer,
    },
    CreatePluginRequest,
    CreatePluginRequestProto,
    CreatePluginResponse,
    CreatePluginResponseProto,
    DeployPluginRequest,
    DeployPluginRequestProto,
    DeployPluginResponse,
    DeployPluginResponseProto,
    GetAnalyzersForTenantRequest,
    GetAnalyzersForTenantRequestProto,
    GetAnalyzersForTenantResponse,
    GetAnalyzersForTenantResponseProto,
    GetGeneratorsForEventSourceRequest,
    GetGeneratorsForEventSourceRequestProto,
    GetGeneratorsForEventSourceResponse,
    GetGeneratorsForEventSourceResponseProto,
    GetPluginRequest,
    GetPluginRequestProto,
    GetPluginResponse,
    GetPluginResponseProto,
    TearDownPluginRequest,
    TearDownPluginRequestProto,
    TearDownPluginResponse,
    TearDownPluginResponseProto,
};
use tonic::{
    transport::Server,
    Request,
    Response,
    Status,
};

#[derive(Debug, thiserror::Error)]
pub enum PluginRegistryServiceError {
    #[error("TODO implement this enum with real error types")]
    TodoImplementThisEnumError(),
}

pub struct PluginRegistry {}

impl PluginRegistry {
    #[allow(dead_code)]
    async fn create_plugin(
        &self,
        _request: CreatePluginRequest,
    ) -> Result<CreatePluginResponse, PluginRegistryServiceError> {
        todo!()
    }

    #[allow(dead_code)]
    async fn get_plugin(
        &self,
        _request: GetPluginRequest,
    ) -> Result<GetPluginResponse, PluginRegistryServiceError> {
        todo!()
    }

    #[allow(dead_code)]
    async fn deploy_plugin(
        &self,
        _request: DeployPluginRequest,
    ) -> Result<DeployPluginResponse, PluginRegistryServiceError> {
        todo!()
    }

    #[allow(dead_code)]
    async fn tear_down_plugin(
        &self,
        _request: TearDownPluginRequest,
    ) -> Result<TearDownPluginResponse, PluginRegistryServiceError> {
        todo!()
    }

    #[allow(dead_code)]
    async fn get_generators_for_event_source(
        &self,
        _request: GetGeneratorsForEventSourceRequest,
    ) -> Result<GetGeneratorsForEventSourceResponse, PluginRegistryServiceError> {
        todo!()
    }

    #[allow(dead_code)]
    async fn get_analyzers_for_tenant(
        &self,
        _request: GetAnalyzersForTenantRequest,
    ) -> Result<GetAnalyzersForTenantResponse, PluginRegistryServiceError> {
        todo!()
    }
}

#[async_trait::async_trait]
impl PluginRegistryService for PluginRegistry {
    async fn create_plugin(
        &self,
        _request: Request<CreatePluginRequestProto>,
    ) -> Result<Response<CreatePluginResponseProto>, Status> {
        todo!()
    }

    async fn get_plugin(
        &self,
        _request: Request<GetPluginRequestProto>,
    ) -> Result<Response<GetPluginResponseProto>, Status> {
        todo!()
    }

    async fn deploy_plugin(
        &self,
        _request: Request<DeployPluginRequestProto>,
    ) -> Result<Response<DeployPluginResponseProto>, Status> {
        todo!()
    }

    async fn tear_down_plugin(
        &self,
        _request: Request<TearDownPluginRequestProto>,
    ) -> Result<Response<TearDownPluginResponseProto>, Status> {
        todo!()
    }

    async fn get_generators_for_event_source(
        &self,
        _request: Request<GetGeneratorsForEventSourceRequestProto>,
    ) -> Result<Response<GetGeneratorsForEventSourceResponseProto>, Status> {
        // Stub implementation, for integration test smoke-test purposes.
        // Replace with a real implementation soon!
        let message = _request.into_inner();
        Ok(Response::new(GetGeneratorsForEventSourceResponseProto {
            plugin_ids: [message.event_source_id].into_iter().flatten().collect(),
        }))
    }

    async fn get_analyzers_for_tenant(
        &self,
        _request: Request<GetAnalyzersForTenantRequestProto>,
    ) -> Result<Response<GetAnalyzersForTenantResponseProto>, Status> {
        todo!()
    }
}

pub async fn exec_service(socket_addr: SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
    let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter
        .set_serving::<PluginRegistryServiceServer<PluginRegistry>>()
        .await;

    let plugin_work_queue = PluginRegistry {};

    tracing::info!(
        message="HealthServer + PluginRegistry listening",
        addr=?socket_addr,
    );

    Server::builder()
        .trace_fn(|request| {
            tracing::info_span!(
                "PluginRegistry",
                headers = ?request.headers(),
                method = ?request.method(),
                uri = %request.uri(),
                extensions = ?request.extensions(),
            )
        })
        .add_service(health_service)
        .add_service(PluginRegistryServiceServer::new(plugin_work_queue))
        .serve(socket_addr)
        .await?;

    Ok(())
}
