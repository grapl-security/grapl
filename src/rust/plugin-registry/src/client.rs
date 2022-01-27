use std::time::Duration;

use rust_proto::plugin_registry::{
    plugin_registry_service_client::PluginRegistryServiceClient as _PluginRegistryServiceClient,
    CreatePluginRequest,
    CreatePluginRequestProto,
    CreatePluginResponse,
    DeployPluginRequest,
    DeployPluginRequestProto,
    DeployPluginResponse,
    GetAnalyzersForTenantRequest,
    GetAnalyzersForTenantRequestProto,
    GetAnalyzersForTenantResponse,
    GetGeneratorsForEventSourceRequest,
    GetGeneratorsForEventSourceRequestProto,
    GetGeneratorsForEventSourceResponse,
    GetPluginRequest,
    GetPluginRequestProto,
    GetPluginResponse,
    PluginRegistryDeserializationError,
    TearDownPluginRequest,
    TearDownPluginRequestProto,
    TearDownPluginResponse,
};
use tonic::{
    codegen::{
        Body,
        StdError,
    },
    transport::Endpoint,
};

const ADDRESS_ENV_VAR: &'static str = "GRAPL_PLUGIN_REGISTRY_ADDRESS";

#[derive(Debug, thiserror::Error)]
pub enum PluginRegistryServiceClientError {
    #[error("ErrorStatus")]
    ErrorStatus(#[from] tonic::Status),
    #[error("PluginRegistryDeserializationError")]
    PluginRegistryDeserializationError(#[from] PluginRegistryDeserializationError),
}

pub struct PluginRegistryServiceClient<T> {
    inner: _PluginRegistryServiceClient<T>,
}

impl PluginRegistryServiceClient<tonic::transport::Channel> {
    /// Create a client from environment
    #[tracing::instrument(err)]
    pub async fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let address = std::env::var(ADDRESS_ENV_VAR).expect(ADDRESS_ENV_VAR);
        Self::from_endpoint(address).await
    }

    /// Create a client from a specific endpoint
    #[tracing::instrument(err)]
    pub async fn from_endpoint(address: String) -> Result<Self, Box<dyn std::error::Error>> {
        tracing::debug!(message = "Connecting to endpoint");

        // TODO: It might make sense to make these values configurable.
        let endpoint = Endpoint::from_shared(address)?
            .timeout(Duration::from_secs(5))
            .concurrency_limit(30);
        let channel = endpoint.connect().await?;
        Ok(Self::new(_PluginRegistryServiceClient::new(
            channel.clone(),
        )))
    }
}

impl<T> PluginRegistryServiceClient<T>
where
    T: tonic::client::GrpcService<tonic::body::BoxBody>,
    T::ResponseBody: Body + Send + 'static,
    T::Error: Into<StdError>,
    <T::ResponseBody as Body>::Error: Into<StdError> + Send,
{
    pub fn new(inner: _PluginRegistryServiceClient<T>) -> Self {
        Self { inner }
    }

    /// create a new plugin
    pub async fn create_plugin(
        &mut self,
        request: CreatePluginRequest,
    ) -> Result<CreatePluginResponse, PluginRegistryServiceClientError> {
        let response = self
            .inner
            .create_plugin(CreatePluginRequestProto::from(request))
            .await?;
        let response = response.into_inner();
        let response = CreatePluginResponse::try_from(response)?;
        Ok(response)
    }
    /// retrieve the plugin corresponding to the given plugin_id
    pub async fn get_plugin(
        &mut self,
        request: GetPluginRequest,
    ) -> Result<GetPluginResponse, PluginRegistryServiceClientError> {
        let response = self
            .inner
            .get_plugin(GetPluginRequestProto::from(request))
            .await?;
        let response = response.into_inner();
        let response = GetPluginResponse::try_from(response)?;
        Ok(response)
    }
    /// turn on a particular plugin's code
    pub async fn deploy_plugin(
        &mut self,
        request: DeployPluginRequest,
    ) -> Result<DeployPluginResponse, PluginRegistryServiceClientError> {
        let response = self
            .inner
            .deploy_plugin(DeployPluginRequestProto::from(request))
            .await?;
        let response = DeployPluginResponse::try_from(response.into_inner())?;
        Ok(response)
    }
    /// turn off a particular plugin's code
    pub async fn tear_down_plugin(
        &mut self,
        request: TearDownPluginRequest,
    ) -> Result<TearDownPluginResponse, PluginRegistryServiceClientError> {
        self.inner
            .tear_down_plugin(TearDownPluginRequestProto::from(request))
            .await?;
        todo!()
    }
    /// Given information about an event source, return all generators that handle that event source
    #[tracing::instrument(skip(self, request), err)]
    pub async fn get_generators_for_event_source(
        &mut self,
        request: GetGeneratorsForEventSourceRequest,
    ) -> Result<GetGeneratorsForEventSourceResponse, PluginRegistryServiceClientError> {
        self.inner
            .get_generators_for_event_source(GetGeneratorsForEventSourceRequestProto::from(request))
            .await?;
        todo!()
    }
    /// Given information about a tenant, return all analyzers for that tenant
    pub async fn get_analyzers_for_tenant(
        &mut self,
        request: GetAnalyzersForTenantRequest,
    ) -> Result<GetAnalyzersForTenantResponse, PluginRegistryServiceClientError> {
        self.inner
            .get_analyzers_for_tenant(GetAnalyzersForTenantRequestProto::from(request))
            .await?;
        todo!()
    }
}
