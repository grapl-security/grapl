use tonic::transport::Endpoint;

use crate::{
    graplinc::grapl::api::{
        client::{
            client_impl,
            Client,
            ClientError,
            Connectable,
        },
        pipeline_ingress::v1beta1 as native,
    },
    protobufs::graplinc::grapl::api::pipeline_ingress::v1beta1::pipeline_ingress_service_client::PipelineIngressServiceClient,
};

#[async_trait::async_trait]
impl Connectable for PipelineIngressServiceClient<tonic::transport::Channel> {
    async fn connect(endpoint: Endpoint) -> Result<Self, ClientError> {
        Ok(Self::connect(endpoint).await?)
    }
}

pub struct PipelineIngressClient {
    client: Client<PipelineIngressServiceClient<tonic::transport::Channel>>,
}

impl client_impl::WithClient<PipelineIngressServiceClient<tonic::transport::Channel>>
    for PipelineIngressClient
{
    const SERVICE_NAME: &'static str =
        "graplinc.grapl.api.pipeline_ingress.v1beta1.PipelineIngressService";

    fn with_client(
        client: Client<PipelineIngressServiceClient<tonic::transport::Channel>>,
    ) -> Self {
        Self { client }
    }
}

impl PipelineIngressClient {
    pub async fn publish_raw_log(
        &mut self,
        request: native::PublishRawLogRequest,
    ) -> Result<native::PublishRawLogResponse, ClientError> {
        self.client
            .execute(
                request,
                |status| status.code() == tonic::Code::Unavailable,
                10,
                |mut client, request| async move { client.publish_raw_log(request).await },
            )
            .await
    }
}
