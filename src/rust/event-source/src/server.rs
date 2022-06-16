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
    pub db_client: EventSourceDbClient,
}

impl EventSourceApiImpl {
    pub async fn try_from(config: &EventSourceConfig) -> Result<Self, EventSourceError> {
        let config = config.clone();
        let db_client = EventSourceDbClient::try_from(config.db_config.clone()).await?;
        Ok(Self { config, db_client })
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
        let created_row = self
            .db_client
            .create_event_source(request.display_name, request.description, request.tenant_id)
            .await?;
        Ok(native::CreateEventSourceResponse {
            event_source_id: created_row.event_source_id,
            created_time: created_row.created_time.into(),
        })
    }

    #[tracing::instrument(skip(self, _request), err)]
    async fn update_event_source(
        &self,
        _request: native::UpdateEventSourceRequest,
    ) -> Result<native::UpdateEventSourceResponse, Self::Error> {
        todo!()
    }

    #[tracing::instrument(skip(self, request), err)]
    async fn get_event_source(
        &self,
        request: native::GetEventSourceRequest,
    ) -> Result<native::GetEventSourceResponse, Self::Error> {
        let row = self
            .db_client
            .get_event_source(request.event_source_id)
            .await?;
        let event_source = native::EventSource {
            tenant_id: row.tenant_id,
            event_source_id: row.event_source_id,
            display_name: row.display_name,
            description: row.description,
            created_time: row.created_time.into(),
            last_updated_time: row.last_updated_time.into(),
            active: row.active,
        };
        Ok(native::GetEventSourceResponse { event_source })
    }
}
