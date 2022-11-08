use tonic::transport::Endpoint;

use crate::{
    graplinc::grapl::api::{
        client::{
            Client,
            ClientError,
            Connectable,
            WithClient,
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

#[derive(Clone)]
pub struct PipelineIngressClient {
    client: Client<PipelineIngressServiceClient<tonic::transport::Channel>>,
}

impl WithClient<PipelineIngressServiceClient<tonic::transport::Channel>> for PipelineIngressClient {
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
