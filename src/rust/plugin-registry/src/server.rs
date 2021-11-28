use rust_proto::plugin_registry::plugin_registry_service_server::PluginRegistryService;

use rust_proto::plugin_registry::{
    _CreatePluginRequest,
    _CreatePluginResponse,
    _DeployPluginRequest,
    _DeployPluginResponse,
    _GetAnalyzersForTenantRequest,
    _GetAnalyzersForTenantResponse,
    _GetGeneratorForEventSourceRequest,
    _GetGeneratorForEventSourceResponse,
    _GetPluginRequest,
    _GetPluginResponse,
    _TearDownPluginRequest,
    _TearDownPluginResponse,
};

use rust_proto::plugin_registry::{
    CreatePluginRequest,
    CreatePluginResponse,
    DeployPluginRequest,
    DeployPluginResponse,
    GetAnalyzersForTenantRequest,
    GetAnalyzersForTenantResponse,
    GetGeneratorForEventSourceRequest,
    GetGeneratorForEventSourceResponse,
    GetPluginRequest,
    GetPluginResponse,
    TearDownPluginRequest,
    TearDownPluginResponse,
};

use tonic::{Request, Response, Status};

#[derive(Debug, thiserror::Error)]
pub enum PluginRegistryServiceError {

}

pub struct PluginRegistry {}

impl PluginRegistry {
    async fn _create_plugin(&self, _request: CreatePluginRequest) -> Result<CreatePluginResponse, PluginRegistryServiceError> {
        todo!()
    }

    async fn _get_plugin(&self, _request: GetPluginRequest) -> Result<GetPluginResponse, PluginRegistryServiceError> {
        todo!()
    }

    async fn _deploy_plugin(&self, _request: DeployPluginRequest) -> Result<DeployPluginResponse, PluginRegistryServiceError> {
        todo!()
    }

    async fn _tear_down_plugin(&self, _request: TearDownPluginRequest) -> Result<TearDownPluginResponse, PluginRegistryServiceError> {
        todo!()
    }

    async fn _get_generator_for_event_source(&self, _request: GetGeneratorForEventSourceRequest) -> Result<GetGeneratorForEventSourceResponse, PluginRegistryServiceError> {
        todo!()
    }

    async fn _get_analyzers_for_tenant(&self, _request: GetAnalyzersForTenantRequest) -> Result<GetAnalyzersForTenantResponse, PluginRegistryServiceError> {
        todo!()
    }

}

#[async_trait::async_trait]
impl PluginRegistryService for PluginRegistry {
    async fn create_plugin(&self, _request: Request<_CreatePluginRequest>) -> Result<Response<_CreatePluginResponse>, Status> {
        todo!()
    }

    async fn get_plugin(&self, _request: Request<_GetPluginRequest>) -> Result<Response<_GetPluginResponse>, Status> {
        todo!()
    }

    async fn deploy_plugin(&self, _request: Request<_DeployPluginRequest>) -> Result<Response<_DeployPluginResponse>, Status> {
        todo!()
    }

    async fn tear_down_plugin(&self, _request: Request<_TearDownPluginRequest>) -> Result<Response<_TearDownPluginResponse>, Status> {
        todo!()
    }

    async fn get_generator_for_event_source(&self, _request: Request<_GetGeneratorForEventSourceRequest>) -> Result<Response<_GetGeneratorForEventSourceResponse>, Status> {
        todo!()
    }

    async fn get_analyzers_for_tenant(&self, _request: Request<_GetAnalyzersForTenantRequest>) -> Result<Response<_GetAnalyzersForTenantResponse>, Status> {
        todo!()
    }
}