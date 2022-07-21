use std::fmt::Debug;

use crate::{
    graplinc::grapl::api::event_source::v1beta1 as native,
    protobufs::graplinc::grapl::api::event_source::v1beta1::{
        self as proto,
        event_source_service_client::EventSourceServiceClient as EventSourceServiceClientProto,
    },
    protocol::{
        endpoint::Endpoint,
        service_client::{
            ConnectError,
            Connectable,
            NamedService,
        },
        status::Status,
    },
    SerDeError,
};

#[derive(Debug, thiserror::Error)]
pub enum EventSourceServiceClientError {
    #[error("ErrorStatus")]
    ErrorStatus(#[from] Status),

    #[error("SerDeError")]
    SerDeError(#[from] SerDeError),
}

impl From<tonic::Status> for EventSourceServiceClientError {
    fn from(tonic_status: tonic::Status) -> Self {
        EventSourceServiceClientError::ErrorStatus(Status::from(tonic_status))
    }
}

#[derive(Clone)]
pub struct EventSourceServiceClient {
    proto_client: EventSourceServiceClientProto<tonic::transport::Channel>,
}

#[async_trait::async_trait]
impl Connectable for EventSourceServiceClient {
    #[tracing::instrument(err)]
    async fn connect(endpoint: Endpoint) -> Result<Self, ConnectError> {
        Ok(EventSourceServiceClient {
            proto_client: EventSourceServiceClientProto::connect(endpoint).await?,
        })
    }
}

impl EventSourceServiceClient {
    pub async fn create_event_source(
        &mut self,
        request: native::CreateEventSourceRequest,
    ) -> Result<native::CreateEventSourceResponse, EventSourceServiceClientError> {
        let response = self
            .proto_client
            .create_event_source(proto::CreateEventSourceRequest::from(request))
            .await?;
        Ok(response.into_inner().try_into()?)
    }

    pub async fn update_event_source(
        &mut self,
        request: native::UpdateEventSourceRequest,
    ) -> Result<native::UpdateEventSourceResponse, EventSourceServiceClientError> {
        let response = self
            .proto_client
            .update_event_source(proto::UpdateEventSourceRequest::from(request))
            .await?;
        Ok(response.into_inner().try_into()?)
    }

    pub async fn get_event_source(
        &mut self,
        request: native::GetEventSourceRequest,
    ) -> Result<native::GetEventSourceResponse, EventSourceServiceClientError> {
        let response = self
            .proto_client
            .get_event_source(proto::GetEventSourceRequest::from(request))
            .await?;
        Ok(response.into_inner().try_into()?)
    }
}

impl NamedService for EventSourceServiceClient {
    const SERVICE_NAME: &'static str = "graplinc.grapl.api.event_source.v1beta1.EventSourceService";
}
