use std::marker::PhantomData;

use futures::TryFutureExt;
use proto::plugin_registry_service_server::PluginRegistryService;
use thiserror::Error;
use tonic::{
    Request,
    Response,
};

use crate::{
    graplinc::grapl::api::plugin_registry::v1beta1::{
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
    },
    protobufs::graplinc::grapl::api::plugin_registry::v1beta1 as proto,
    SerDeError,
};

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum PluginRegistryApiError {
    #[error("failed to serialize/deserialize {0}")]
    SerDeError(#[from] SerDeError),

    #[error("received unfavorable gRPC status {0}")]
    GrpcStatus(#[from] tonic::Status),
}

//impl From<PluginRegistryServiceClientError> for

/// Implement this trait to define the API business logic
#[tonic::async_trait]
pub trait PluginRegistryApi<E>
where
    E: ToString + 'static,
{
    async fn create_plugin(&self, request: CreatePluginRequest) -> Result<CreatePluginResponse, E>;

    async fn get_plugin(&self, request: GetPluginRequest) -> Result<GetPluginResponse, E>;

    async fn deploy_plugin(&self, request: DeployPluginRequest) -> Result<DeployPluginResponse, E>;

    async fn tear_down_plugin(
        &self,
        _request: TearDownPluginRequest,
    ) -> Result<TearDownPluginResponse, E>;

    async fn get_generators_for_event_source(
        &self,
        _request: GetGeneratorsForEventSourceRequest,
    ) -> Result<GetGeneratorsForEventSourceResponse, E>;

    async fn get_analyzers_for_tenant(
        &self,
        _request: GetAnalyzersForTenantRequest,
    ) -> Result<GetAnalyzersForTenantResponse, E>;
}

/// This struct implements the internal gRPC representation of the server.
/// We've implemented the service trait generated by tonic
/// in such a way that it delegates to an externally supplied
/// PluginRegistryApi. This way all the protocol buffer compiler generated
/// types are encapsulated, and the public API is implemented in terms of
/// this crate's sanitized types.
struct PluginRegistryProto<T, E>
where
    T: PluginRegistryApi<E>,
    E: ToString + 'static,
{
    api_server: T,
    _e: PhantomData<E>,
}

#[tonic::async_trait]
impl<T, E> PluginRegistryService for PluginRegistryProto<T, E>
where
    T: PluginRegistryApi<E> + Send + Sync + 'static,
    E: ToString + Send + Sync + 'static,
{
    async fn create_plugin(
        &self,
        request: Request<proto::CreatePluginRequest>,
    ) -> Result<Response<proto::CreatePluginResponse>, tonic::Status> {
        let inner_request: CreatePluginRequest = request
            .into_inner()
            .try_into()
            .map_err(|e: SerDeError| tonic::Status::unknown(e.to_string()))?;

        let response = self
            .api_server
            .create_plugin(inner_request)
            .map_err(|e| tonic::Status::unknown(e.to_string()))
            .await?;

        Ok(Response::new(response.into()))
    }

    async fn get_plugin(
        &self,
        request: Request<proto::GetPluginRequest>,
    ) -> Result<Response<proto::GetPluginResponse>, tonic::Status> {
        let inner_request: GetPluginRequest = request
            .into_inner()
            .try_into()
            .map_err(|e: SerDeError| tonic::Status::unknown(e.to_string()))?;

        let response = self
            .api_server
            .get_plugin(inner_request)
            .map_err(|e| tonic::Status::unknown(e.to_string()))
            .await?;

        Ok(Response::new(response.into()))
    }

    async fn deploy_plugin(
        &self,
        request: Request<proto::DeployPluginRequest>,
    ) -> Result<Response<proto::DeployPluginResponse>, tonic::Status> {
        let inner_request: DeployPluginRequest = request
            .into_inner()
            .try_into()
            .map_err(|e: SerDeError| tonic::Status::unknown(e.to_string()))?;

        let response = self
            .api_server
            .deploy_plugin(inner_request)
            .map_err(|e| tonic::Status::unknown(e.to_string()))
            .await?;

        Ok(Response::new(response.into()))
    }

    async fn tear_down_plugin(
        &self,
        _request: Request<proto::TearDownPluginRequest>,
    ) -> Result<Response<proto::TearDownPluginResponse>, tonic::Status> {
        todo!()
    }

    #[tracing::instrument(skip(self, _request), err)]
    async fn get_generators_for_event_source(
        &self,
        _request: Request<proto::GetGeneratorsForEventSourceRequest>,
    ) -> Result<Response<proto::GetGeneratorsForEventSourceResponse>, tonic::Status> {
        todo!()
    }

    async fn get_analyzers_for_tenant(
        &self,
        _request: Request<proto::GetAnalyzersForTenantRequest>,
    ) -> Result<Response<proto::GetAnalyzersForTenantResponse>, tonic::Status> {
        todo!()
    }
}
