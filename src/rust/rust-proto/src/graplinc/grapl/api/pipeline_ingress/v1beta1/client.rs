use std::time::Duration;

use client_executor::strategy::FibonacciBackoff;
use tonic::transport::Endpoint;

use crate::{
    graplinc::grapl::api::{
        pipeline_ingress::v1beta1 as native,
        client::{
            Connectable,
            Client,
            ClientError,
            Configuration,
        },
    },
    protobufs::graplinc::grapl::api::pipeline_ingress::v1beta1::pipeline_ingress_service_client::PipelineIngressServiceClient,
};

#[async_trait::async_trait]
impl Connectable for PipelineIngressServiceClient<tonic::transport::Channel> {
    async fn connect(endpoint: Endpoint) -> Result<Self, ClientError> {
        Ok(Self::connect(endpoint).await?)
    }
}

pub struct PipelineIngressClient<B>
where
    B: IntoIterator<Item = Duration> + Clone,
{
    client: Client<B, PipelineIngressServiceClient<tonic::transport::Channel>>,
}

impl <B> PipelineIngressClient<B>
where
    B: IntoIterator<Item = Duration> + Clone,
{
    const SERVICE_NAME: &'static str =
        "graplinc.grapl.api.pipeline_ingress.v1beta1.PipelineIngressService";

    pub fn new<A>(
        address: A,
        request_timeout: Duration,
        executor_timeout: Duration,
        concurrency_limit: usize,
        initial_backoff_delay: Duration,
        maximum_backoff_delay: Duration,
    ) -> Result<Self, ClientError>
    where
        A: TryInto<Endpoint>,
    {
        let configuration = Configuration::new(
            Self::SERVICE_NAME,
            address,
            request_timeout,
            executor_timeout,
            concurrency_limit,
            FibonacciBackoff::from_millis(initial_backoff_delay.as_millis())
                .max_delay(maximum_backoff_delay)
                .map(client_executor::strategy::jitter),
        )?;
        let client = Client::new(configuration);

        Ok(Self { client })
    }

    pub async fn publish_raw_log(
        &mut self,
        request: native::PublishRawLogRequest,
    ) -> Result<native::PublishRawLogResponse, ClientError> {
        Ok(self.client.execute(
            request,
            |status, request| status.code() == tonic::Code::Unavailable,
            10,
            |client, request| client.publish_raw_log(request),
        ).await?)
    }
}
