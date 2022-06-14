use std::time::Duration;

use rust_proto_new::{
    graplinc::grapl::api::event_source::{
        v1beta1 as native,
        v1beta1::server::{
            EventSourceApi,
            EventSourceServer,
        },
    },
    protocol::healthcheck::HealthcheckStatus,
};
use tokio::net::TcpListener;

use crate::{
    config::EventSourceConfig,
    db::EventSourceDbClient,
    error::EventSourceError,
};

pub async fn exec_service(config: EventSourceConfig) -> Result<(), Box<dyn std::error::Error>> {
    let api_implementor = EventSourceApiImpl::try_from(&config).await?;
    let (server, _shutdown_tx) = EventSourceServer::new(
        api_implementor,
        TcpListener::bind(
            config
                .service_config
                .event_source_service_bind_address
                .clone(),
        )
        .await?,
        || async { Ok(HealthcheckStatus::Serving) }, // FIXME: this is garbage
        Duration::from_millis(
            config
                .service_config
                .event_source_service_healthcheck_polling_interval_ms,
        ),
    );
    tracing::info!(
        message = "starting gRPC server",
        socket_address = %config.service_config.event_source_service_bind_address,
    );

    server.serve().await
}

pub struct EventSourceApiImpl {
    pub config: EventSourceConfig,
    pub db: EventSourceDbClient,
}

impl EventSourceApiImpl {
    pub async fn try_from(config: &EventSourceConfig) -> Result<Self, EventSourceError> {
        let config = config.clone();
        let db = EventSourceDbClient::try_from(config.db_config.clone()).await?;
        Ok(Self { config, db })
    }
}

#[async_trait::async_trait]
impl EventSourceApi for EventSourceApiImpl {
    type Error = EventSourceError;

    #[tracing::instrument(skip(self, request), err)]
    async fn create_event_source(
        &self,
        request: native::CreateEventSourceRequest,
    ) -> Result<native::CreateEventSourceResponse, Self::Error> {
        let _ = request;
        unreachable!()
    }

    #[tracing::instrument(skip(self, request), err)]
    async fn update_event_source(
        &self,
        request: native::UpdateEventSourceRequest,
    ) -> Result<native::UpdateEventSourceResponse, Self::Error> {
        let _ = request;
        unreachable!()
    }

    #[tracing::instrument(skip(self, request), err)]
    async fn get_event_source(
        &self,
        request: native::GetEventSourceRequest,
    ) -> Result<native::GetEventSourceResponse, Self::Error> {
        let _ = request;
        unreachable!()
    }
}
