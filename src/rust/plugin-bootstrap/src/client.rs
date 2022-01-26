use rust_proto::plugin_bootstrap::{
    GetBootstrapInfoRequest,
    GetBootstrapInfoRequestProto,
    GetBootstrapInfoResponse,
    PluginBootstrapDeserializationError,
    PluginBootstrapServiceClient as PluginBootstrapClientProto,
};
use tonic::{
    codegen::{
        Body,
        StdError,
    },
    transport::Endpoint,
};

const ADDRESS_ENV_VAR: &str = "GRAPL_PLUGIN_BOOTSTRAP_ADDRESS";

#[derive(Debug, thiserror::Error)]
pub enum PluginBootstrapClientError {
    #[error("GrpcStatus {0}")]
    Status(#[from] tonic::Status),
    #[error("PluginBootstrapDeserializationError {0}")]
    PluginBootstrapDeserializationError(#[from] PluginBootstrapDeserializationError),
}

#[derive(Debug, Clone)]
pub struct PluginBootstrapClient<T> {
    inner: PluginBootstrapClientProto<T>,
}

impl PluginBootstrapClient<tonic::transport::Channel> {
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
            .timeout(std::time::Duration::from_secs(5))
            .concurrency_limit(30);
        let channel = endpoint.connect().await?;
        Ok(Self::new(PluginBootstrapClientProto::new(channel)))
    }
}

impl<T> PluginBootstrapClient<T>
where
    T: tonic::client::GrpcService<tonic::body::BoxBody>,
    T::ResponseBody: Body + Send + 'static,
    T::Error: Into<StdError>,
    <T::ResponseBody as Body>::Error: Into<StdError> + Send,
{
    pub fn new(inner: PluginBootstrapClientProto<T>) -> Self {
        Self { inner }
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_bootstrap_info(
        &mut self,
        request: GetBootstrapInfoRequest,
    ) -> Result<GetBootstrapInfoResponse, PluginBootstrapClientError> {
        let response = self
            .inner
            .get_bootstrap_info(GetBootstrapInfoRequestProto::from(request))
            .await?;
        let response = response.into_inner();
        Ok(response.try_into()?)
    }
}
