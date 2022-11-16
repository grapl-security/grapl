use tonic::transport::Endpoint;
use tracing::instrument;

use crate::{
    graplinc::grapl::api::{
        client::{
            Client,
            ClientError,
            Connectable,
            WithClient,
        },
        plugin_sdk::analyzers::v1beta1::messages as native,
        request_metadata::RequestMetadata,
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

impl WithClient<AnalyzerServiceClient<tonic::transport::Channel>> for AnalyzerClient {
    fn with_client(client: Client<AnalyzerServiceClient<tonic::transport::Channel>>) -> Self {
        Self { client }
    }
}

impl AnalyzerClient {
    /// retrieve the plugin corresponding to the given plugin_id
    #[instrument(skip(self, request, request_metadata), err)]
    pub async fn run_analyzer(
        &mut self,
        request: native::RunAnalyzerRequest,
        request_metadata: Option<RequestMetadata>,
    ) -> Result<native::RunAnalyzerResponse, ClientError> {
        self.client
            .execute(
                request,
                request_metadata,
                |status| status.code() == tonic::Code::Unavailable,
                10,
                |mut client, request| async move { client.run_analyzer(request).await },
            )
            .await
    }
}
