use std::fmt::Debug;

use event_source_service_client::EventSourceServiceClient as EventSourceServiceClientProto;

pub use crate::protobufs::graplinc::grapl::api::event_source::v1beta1::event_source_service_client;
use crate::{
    graplinc::grapl::api::event_source::v1beta1 as native,
    protobufs::graplinc::grapl::api::event_source::v1beta1 as proto,
    protocol::{
        status::Status,
        tls::ClientTlsConfig, service_client::NamedService,
    },
    SerDeError,
};

#[derive(Debug, thiserror::Error)]
pub enum EventSourceServiceClientError {
    #[error(transparent)]
    TransportError(#[from] tonic::transport::Error),
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

impl EventSourceServiceClient {
    #[tracing::instrument(skip(tls_config), err)]
    pub async fn connect<T>(
        endpoint: T,
        tls_config: Option<ClientTlsConfig>,
    ) -> Result<Self, EventSourceServiceClientError>
    where
        T: std::convert::TryInto<tonic::transport::Endpoint, Error = tonic::transport::Error>
            + Debug,
        T::Error: std::error::Error + Send + Sync + 'static,
    {
        let mut endpoint: tonic::transport::Endpoint = endpoint.try_into()?;
        if let Some(inner_config) = tls_config {
            endpoint = endpoint.tls_config(inner_config.into())?;
        }
        Ok(Self {
            proto_client: EventSourceServiceClientProto::connect(endpoint).await?,
        })
    }

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
    const SERVICE_NAME: &'static str =
        "graplinc.grapl.api.event_source.v1beta1.EventSourceService";
}