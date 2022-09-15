use std::time::Duration;

use client_executor::{
    Executor,
    ExecutorConfig,
};

use crate::{
    client_factory::services::EventSourceClientConfig,
    client_macros::RpcConfig,
    create_proto_client,
    execute_client_rpc,
    graplinc::grapl::api::event_source::v1beta1 as native,
    protobufs::graplinc::grapl::api::event_source::v1beta1::{
        self as proto,
        event_source_service_client::EventSourceServiceClient as EventSourceServiceClientProto,
    },
    protocol::{
        endpoint::Endpoint,
        error::GrpcClientError,
        service_client::{
            ConfigConnectable,
            ConnectError,
            Connectable,
        },
    },
};

pub type EventSourceServiceClientError = GrpcClientError;

#[derive(Clone)]
pub struct EventSourceServiceClient {
    proto_client: EventSourceServiceClientProto<tonic::transport::Channel>,
    executor: Executor,
}

#[async_trait::async_trait]
impl Connectable for EventSourceServiceClient {
    const SERVICE_NAME: &'static str = "graplinc.grapl.api.event_source.v1beta1.EventSourceService";

    #[tracing::instrument(err)]
    async fn connect_with_endpoint(endpoint: Endpoint) -> Result<Self, ConnectError> {
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

impl ConfigConnectable for EventSourceServiceClient {
    type Config = EventSourceClientConfig;
}

impl EventSourceServiceClient {
    #[tracing::instrument(skip(self, request), err)]
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
            RpcConfig::default(),
        )
    }

    #[tracing::instrument(skip(self, request), err)]
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
            RpcConfig::default(),
        )
    }

    #[tracing::instrument(skip(self, request), err)]
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
            RpcConfig::default(),
        )
    }
}
