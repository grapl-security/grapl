use tonic::transport::Endpoint;

use crate::{
    graplinc::grapl::api::{
        client::{
            client_impl,
            Client,
            ClientError,
            Connectable,
        },
        event_source::v1beta1 as native,
    },
    protobufs::graplinc::grapl::api::event_source::v1beta1::event_source_service_client::EventSourceServiceClient,
};

#[async_trait::async_trait]
impl Connectable for EventSourceServiceClient<tonic::transport::Channel> {
    async fn connect(endpoint: Endpoint) -> Result<Self, ClientError> {
        Ok(Self::connect(endpoint).await?)
    }
}

#[derive(Clone)]
pub struct EventSourceClient {
    client: Client<EventSourceServiceClient<tonic::transport::Channel>>,
}

impl client_impl::WithClient<EventSourceServiceClient<tonic::transport::Channel>>
    for EventSourceClient
{
    const SERVICE_NAME: &'static str = "graplinc.grapl.api.event_source.v1beta1.EventSourceService";

    fn with_client(client: Client<EventSourceServiceClient<tonic::transport::Channel>>) -> Self {
        Self { client }
    }
}

impl EventSourceClient {
    #[tracing::instrument(skip(self, request), err)]
    pub async fn create_event_source(
        &mut self,
        request: native::CreateEventSourceRequest,
    ) -> Result<native::CreateEventSourceResponse, ClientError> {
        self.client
            .execute(
                request,
                |status| status.code() == tonic::Code::Unavailable,
                10,
                |mut client, request| async move { client.create_event_source(request).await },
            )
            .await
    }

    #[tracing::instrument(skip(self, request), err)]
    pub async fn update_event_source(
        &mut self,
        request: native::UpdateEventSourceRequest,
    ) -> Result<native::UpdateEventSourceResponse, ClientError> {
        self.client
            .execute(
                request,
                |status| status.code() == tonic::Code::Unavailable,
                10,
                |mut client, request| async move { client.update_event_source(request).await },
            )
            .await
    }

    #[tracing::instrument(skip(self, request), err)]
    pub async fn get_event_source(
        &mut self,
        request: native::GetEventSourceRequest,
    ) -> Result<native::GetEventSourceResponse, ClientError> {
        self.client
            .execute(
                request,
                |status| status.code() == tonic::Code::Unavailable,
                10,
                |mut client, request| async move { client.get_event_source(request).await },
            )
            .await
    }
}
