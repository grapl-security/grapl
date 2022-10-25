use tonic::transport::Endpoint;
use tracing::instrument;

use crate::{
    graplinc::grapl::api::{
        client::{
            client_impl,
            Client,
            ClientError,
            Connectable,
        },
        plugin_sdk::analyzers::v1beta1::messages as native,
    },
    protobufs::graplinc::grapl::api::plugin_sdk::analyzers::v1beta1::analyzer_service_client::AnalyzerServiceClient,
};

#[async_trait::async_trait]
impl Connectable for AnalyzerServiceClient<tonic::transport::Channel> {
    async fn connect(endpoint: Endpoint) -> Result<Self, ClientError> {
        Ok(Self::connect(endpoint).await?)
    }
}

#[derive(Clone)]
pub struct AnalyzerClient {
    client: Client<AnalyzerServiceClient<tonic::transport::Channel>>,
}

impl client_impl::WithClient<AnalyzerServiceClient<tonic::transport::Channel>> for AnalyzerClient {
    const SERVICE_NAME: &'static str = "graplinc.grapl.api.plugin_registry.v1beta1.AnalyzerService";

    fn with_client(client: Client<AnalyzerServiceClient<tonic::transport::Channel>>) -> Self {
        Self { client }
    }
}

impl AnalyzerClient {
    /// retrieve the plugin corresponding to the given plugin_id
    #[instrument(skip(self, request), err)]
    pub async fn run_analyzer(
        &mut self,
        request: native::RunAnalyzerRequest,
    ) -> Result<native::RunAnalyzerResponse, ClientError> {
        self.client
            .execute(
                request,
                |status| status.code() == tonic::Code::Unavailable,
                10,
                |mut client, request| async move { client.run_analyzer(request).await },
            )
            .await
    }
}
