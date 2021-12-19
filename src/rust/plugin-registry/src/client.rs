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

const HOST_ENV_VAR: &'static str = "GRAPL_PLUGIN_REGISTRY_HOST";
const PORT_ENV_VAR: &'static str = "GRAPL_PLUGIN_REGISTRY_PORT";

#[derive(Debug, thiserror::Error)]
pub enum PluginRegistryServiceClientError {
    #[error("Unknown Error")]
    Unknown,
}

pub struct PluginRegistryServiceClient<T> {
    inner: _PluginRegistryServiceClient<T>,
}

impl PluginRegistryServiceClient<tonic::transport::Channel> {
    /// Create a client from environment
    #[tracing::instrument(err)]
    pub async fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let host = std::env::var(HOST_ENV_VAR).expect(HOST_ENV_VAR);
        let port = std::env::var(PORT_ENV_VAR).expect(PORT_ENV_VAR);
        Self::from_endpoint(host, port).await
    }

    /// Create a client from a specific endpoint
    #[tracing::instrument(err)]
    pub async fn from_endpoint(
        host: String,
        port: String,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let endpoint_str = format!("http://{}:{}", host, port);

        tracing::debug!(message = "Connecting to endpoint");

        // TODO: It might make sense to make these values configurable.
        let endpoint = Endpoint::from_shared(endpoint_str)?
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
        let response = self.inner
            .create_plugin(CreatePluginRequestProto::from(request))
            .await
            .expect("todo");
        let response = response.into_inner();
        let response = CreatePluginResponse::try_from(response).expect("todo");
        Ok(response)
    }
    /// retrieve the plugin corresponding to the given plugin_id
    pub async fn get_plugin(
        &mut self,
        request: GetPluginRequest,
    ) -> Result<GetPluginResponse, PluginRegistryServiceClientError> {
        let response = self.inner
            .get_plugin(GetPluginRequestProto::from(request))
            .await
            .expect("todo");
        let response = response.into_inner();
        let response = GetPluginResponse::try_from(response).expect("todo");
        Ok(response)
    }
    /// turn on a particular plugin's code
    pub async fn deploy_plugin(
        &mut self,
        request: DeployPluginRequest,
    ) -> Result<DeployPluginResponse, PluginRegistryServiceClientError> {
        self.inner
            .deploy_plugin(DeployPluginRequestProto::from(request))
            .await
            .expect("todo");
        todo!()
    }
    /// turn off a particular plugin's code
    pub async fn tear_down_plugin(
        &mut self,
        request: TearDownPluginRequest,
    ) -> Result<TearDownPluginResponse, PluginRegistryServiceClientError> {
        self.inner
            .tear_down_plugin(TearDownPluginRequestProto::from(request))
            .await
            .expect("todo");
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
            .await
            .map(|resp| resp.into_inner().try_into().expect("proto to rs"))
            .map_err(|status| {
                tracing::warn!(
                    message="Failed to get_generators_for_event_source",
                    error=?status
                );
                PluginRegistryServiceClientError::Unknown
            })
    }
    /// Given information about a tenant, return all analyzers for that tenant
    pub async fn get_analyzers_for_tenant(
        &mut self,
        request: GetAnalyzersForTenantRequest,
    ) -> Result<GetAnalyzersForTenantResponse, PluginRegistryServiceClientError> {
        self.inner
            .get_analyzers_for_tenant(GetAnalyzersForTenantRequestProto::from(request))
            .await
            .expect("todo");
        todo!()
    }
}
