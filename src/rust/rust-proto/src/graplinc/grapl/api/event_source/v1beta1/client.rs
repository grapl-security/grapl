use std::time::Duration;

use client_executor::strategy::FibonacciBackoff;
use tonic::transport::Endpoint;

use crate::{
    graplinc::grapl::api::{
        event_source::v1beta1 as native,
        client::{
            Client,
            ClientError,
            Configuration,
            Connectable
        },
    },
    protobufs::graplinc::grapl::api::event_source::v1beta1::event_source_service_client::EventSourceServiceClient,
};

#[async_trait::async_trait]
impl Connectable
    for EventSourceServiceClient<tonic::transport::Channel>
{
    async fn connect(endpoint: Endpoint) -> Result<Self, ClientError> {
        Ok(Self::connect(endpoint).await?)
    }
}

#[derive(Clone)]
pub struct EventSourceClient<B>
where
    B: IntoIterator<Item = Duration> + Clone,
{
    client: Client<B, EventSourceServiceClient<tonic::transport::Channel>>,
}

impl <B> EventSourceClient<B>
where
    B: IntoIterator<Item = Duration> + Clone,
{
    const SERVICE_NAME: &'static str = "graplinc.grapl.api.event_source.v1beta1.EventSourceService";

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
        let client = Client::new(configuration)?;

        Ok(Self { client })
    }

    #[tracing::instrument(skip(self, request), err)]
    pub async fn create_event_source(
        &mut self,
        request: native::CreateEventSourceRequest,
    ) -> Result<native::CreateEventSourceResponse, ClientError> {
        Ok(self.client.execute(
            request,
            |status, request| status.code() == tonic::Code::Unavailable,
            10,
            |client, request| client.create_event_source(request),
        ).await?)
    }

    #[tracing::instrument(skip(self, request), err)]
    pub async fn update_event_source(
        &mut self,
        request: native::UpdateEventSourceRequest,
    ) -> Result<native::UpdateEventSourceResponse, ClientError> {
        Ok(self.client.execute(
            request,
            |status, request| status.code() == tonic::Code::Unavailable,
            10,
            |client, request| client.update_event_source(request),
        ).await?)
    }

    #[tracing::instrument(skip(self, request), err)]
    pub async fn get_event_source(
        &mut self,
        request: native::GetEventSourceRequest,
    ) -> Result<native::GetEventSourceResponse, ClientError> {
        Ok(self.client.execute(
            request,
            |status, request| status.code() == tonic::Code::Unavailable,
            10,
            |client, request| client.get_event_source(request),
        ).await?)
    }
}
