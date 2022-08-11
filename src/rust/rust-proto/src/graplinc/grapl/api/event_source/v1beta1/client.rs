use std::{time::Duration};

use client_executor::{Executor, ExecutorConfig};

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
        },
        error::GrpcClientError,
    },
    create_proto_client, execute_client_rpc,
};

type EventSourceServiceClientError = GrpcClientError;

#[derive(Clone)]
pub struct EventSourceServiceClient {
    proto_client: EventSourceServiceClientProto<tonic::transport::Channel>,
    executor: Executor,
}

#[async_trait::async_trait]
impl Connectable for EventSourceServiceClient {
    const SERVICE_NAME: &'static str = "graplinc.grapl.api.event_source.v1beta1.EventSourceService";

    #[tracing::instrument(err)]
    async fn connect(endpoint: Endpoint) -> Result<Self, ConnectError> {
        let executor = Executor::new(ExecutorConfig::new(Duration::from_secs(30)));
        let proto_client = create_proto_client!(
            executor,
            EventSourceServiceClientProto<tonic::transport::Channel>,
            endpoint,
        );

        Ok(EventSourceServiceClient {
            executor,
            proto_client,
        })
    }
}

impl EventSourceServiceClient {
    pub async fn create_event_source(
        &mut self,
        request: native::CreateEventSourceRequest,
    ) -> Result<native::CreateEventSourceResponse, EventSourceServiceClientError> {
        execute_client_rpc!(
            self,
            request,
            create_event_source,
            proto::CreateEventSourceRequest,
            native::CreateEventSourceResponse,
        )
    }

    pub async fn update_event_source(
        &mut self,
        request: native::UpdateEventSourceRequest,
    ) -> Result<native::UpdateEventSourceResponse, EventSourceServiceClientError> {
        execute_client_rpc!(
            self,
            request,
            update_event_source,
            proto::UpdateEventSourceRequest,
            native::UpdateEventSourceResponse,
        )
    }

    pub async fn get_event_source(
        &mut self,
        request: native::GetEventSourceRequest,
    ) -> Result<native::GetEventSourceResponse, EventSourceServiceClientError> {
        execute_client_rpc!(
            self,
            request,
            get_event_source,
            proto::GetEventSourceRequest,
            native::GetEventSourceResponse,
        )
    }
}
