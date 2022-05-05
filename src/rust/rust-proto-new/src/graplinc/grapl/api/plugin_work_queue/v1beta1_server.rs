use crate::graplinc::grapl::api::plugin_registry::v1beta1 as native;

/// Implement this trait to define the API business logic
#[tonic::async_trait]
pub trait PluginWorkQueueApi<E>
where
    E: Into<tonic::Status>,
{
    /*
    async fn create_plugin(&self, request: CreatePluginRequest) -> Result<CreatePluginResponse, E>;

    async fn get_plugin(&self, request: GetPluginRequest) -> Result<GetPluginResponse, E>;

    async fn deploy_plugin(&self, request: DeployPluginRequest) -> Result<DeployPluginResponse, E>;

    async fn tear_down_plugin(
        &self,
        request: TearDownPluginRequest,
    ) -> Result<TearDownPluginResponse, E>;

    async fn get_generators_for_event_source(
        &self,
        request: GetGeneratorsForEventSourceRequest,
    ) -> Result<GetGeneratorsForEventSourceResponse, E>;

    async fn get_analyzers_for_tenant(
        &self,
        request: GetAnalyzersForTenantRequest,
    ) -> Result<GetAnalyzersForTenantResponse, E>;
    */
}