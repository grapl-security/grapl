use tonic::transport::Endpoint;

use crate::{
    graplinc::grapl::api::{
        client::{
            client_impl,
            Client,
            ClientError,
            Connectable,
        },
        plugin_sdk::generators::v1beta1 as native,
    },
    protobufs::graplinc::grapl::api::plugin_sdk::generators::v1beta1::generator_service_client::GeneratorServiceClient,
};

#[async_trait::async_trait]
impl Connectable for GeneratorServiceClient<tonic::transport::Channel> {
    async fn connect(endpoint: Endpoint) -> Result<Self, ClientError> {
        Ok(Self::connect(endpoint).await?)
    }
}

#[derive(Clone)]
pub struct GeneratorClient {
    client: Client<GeneratorServiceClient<tonic::transport::Channel>>,
}

impl client_impl::WithClient<GeneratorServiceClient<tonic::transport::Channel>>
    for GeneratorClient
{
    const SERVICE_NAME: &'static str =
        "graplinc.grapl.api.plugin_sdk.generators.v1beta1.GeneratorService";

    fn with_client(client: Client<GeneratorServiceClient<tonic::transport::Channel>>) -> Self {
        Self { client }
    }
}

impl GeneratorClient {
    #[tracing::instrument(skip(self, request), err)]
    pub async fn run_generator(
        &mut self,
        request: native::RunGeneratorRequest,
    ) -> Result<native::RunGeneratorResponse, ClientError> {
        self.client
            .execute(
                request,
                |status| status.code() == tonic::Code::Unavailable,
                10,
                |mut client, request| async move { client.run_generator(request).await },
            )
            .await
    }
}
