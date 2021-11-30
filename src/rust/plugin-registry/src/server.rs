use rust_proto::plugin_registry::{
    CreatePluginRequestProto,
    CreatePluginResponseProto,
    DeployPluginRequestProto,
    DeployPluginResponseProto,
    GetAnalyzersForTenantRequestProto,
    GetAnalyzersForTenantResponseProto,
    GetGeneratorsForEventSourceRequestProto,
    GetGeneratorsForEventSourceResponseProto,
    GetPluginRequestProto,
    GetPluginResponseProto,
    TearDownPluginRequestProto,
    TearDownPluginResponseProto,
    plugin_registry_service_server::PluginRegistryService,
    CreatePluginRequest,
    CreatePluginResponse,
    DeployPluginRequest,
    DeployPluginResponse,
    GetAnalyzersForTenantRequest,
    GetAnalyzersForTenantResponse,
    GetGeneratorsForEventSourceRequest,
    GetGeneratorsForEventSourceResponse,
    GetPluginRequest,
    GetPluginResponse,
    TearDownPluginRequest,
    TearDownPluginResponse,
};
use tonic::{
    Request,
    Response,
    Status,
};

#[derive(Debug, thiserror::Error)]
pub enum PluginRegistryServiceError {}

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
    async fn get_generator_for_event_source(
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

    async fn get_generator_for_event_source(
        &self,
        _request: Request<GetGeneratorsForEventSourceRequestProto>,
    ) -> Result<Response<GetGeneratorsForEventSourceResponseProto>, Status> {
        todo!()
    }

    async fn get_analyzers_for_tenant(
        &self,
        _request: Request<GetAnalyzersForTenantRequestProto>,
    ) -> Result<Response<GetAnalyzersForTenantResponseProto>, Status> {
        todo!()
    }
}
