use std::{
    env::VarError,
    num::ParseIntError,
    time::Duration,
};

use clap::Parser;
use grapl_tracing::{
    setup_tracing,
    SetupTracingError,
};
use kafka::{
    config::ProducerConfig,
    ConfigurationError as KafkaConfigurationError,
    Producer,
    ProducerError,
};
use rust_proto::{
    graplinc::grapl::{
        api::pipeline_ingress::v1beta1::{
            server::{
                PipelineIngressApi,
                PipelineIngressServer,
            },
            PublishRawLogRequest,
            PublishRawLogResponse,
        },
        pipeline::v1beta1::{
            Envelope,
            RawLog,
        },
    },
    protocol::{
        error::ServeError,
        healthcheck::HealthcheckStatus,
        status::Status,
    },
};
use thiserror::Error;
use tokio::net::TcpListener;
use uuid::Uuid;

#[non_exhaustive]
#[derive(Debug, Error)]
enum IngressApiError {
    #[error("failed to send message to kafka {0}")]
    ProducerError(#[from] ProducerError),
}

impl From<IngressApiError> for Status {
    fn from(e: IngressApiError) -> Self {
        Status::internal(e.to_string())
    }
}

struct IngressApi {
    producer: Producer<RawLog>,
}

impl IngressApi {
    fn new(producer: Producer<RawLog>) -> Self {
        IngressApi { producer }
    }
}

#[async_trait::async_trait]
impl PipelineIngressApi for IngressApi {
    type Error = IngressApiError;

    #[tracing::instrument(skip(self))]
    async fn publish_raw_log(
        &self,
        request: PublishRawLogRequest,
    ) -> Result<PublishRawLogResponse, Self::Error> {
        let tenant_id = request.tenant_id;
        let event_source_id = request.event_source_id;
        // TODO: trace_id should be generated at the edge. This service is
        // currently "the edge" but that won't be true forever. When there is an
        // actual edge service, that service should be responsible for
        // generating the trace_id.
        let trace_id = Uuid::new_v4();

        tracing::debug!(
            message = "publishing raw log",
            tenant_id =% tenant_id,
            event_source_id =% event_source_id,
            trace_id =% trace_id,
        );

        self.producer
            .send(Envelope::new(
                tenant_id,
                trace_id,
                event_source_id,
                RawLog::new(request.log_event),
            ))
            .await?;

        Ok(PublishRawLogResponse::ok())
    }
}

#[non_exhaustive]
#[derive(Debug, Error)]
enum ConfigurationError {
    #[error("failed to configure kafka client {0}")]
    Kafka(#[from] KafkaConfigurationError),

    #[error("ServeError {0}")]
    ServeError(#[from] ServeError),

    #[error("missing environment variable {0}")]
    EnvironmentVariable(#[from] VarError),

    #[error("failed to bind socket address {0}")]
    SocketError(#[from] std::io::Error),

    #[error("failed to parse integer value {0}")]
    ParseInt(#[from] ParseIntError),

    #[error("failed to configure tracing {0}")]
    SetupTracingError(#[from] SetupTracingError),
}

#[tracing::instrument(err)]
async fn handler() -> Result<(), ConfigurationError> {
    let socket_address = std::env::var("PIPELINE_INGRESS_BIND_ADDRESS")?;

    let healthcheck_polling_interval_ms =
        std::env::var("PIPELINE_INGRESS_HEALTHCHECK_POLLING_INTERVAL_MS")?.parse()?;

    let producer_config = ProducerConfig::parse();

    tracing::info!(
        message = "configuring kafka producer",
        producer_config = ?producer_config,
    );
    let producer: Producer<RawLog> = Producer::new(producer_config)?;
    tracing::info!(message = "kafka producer configured successfully",);

    tracing::info!(
        message = "configuring gRPC server",
        socket_address = %socket_address,
    );
    let (server, _shutdown_tx) = PipelineIngressServer::new(
        IngressApi::new(producer),
        TcpListener::bind(socket_address.clone()).await?,
        || async { Ok(HealthcheckStatus::Serving) }, // FIXME: this is garbage
        Duration::from_millis(healthcheck_polling_interval_ms),
    );
    tracing::info!(
        message = "gRPC server configured successfully",
        socket_address = %socket_address,
        service_name = %server.service_name(),
    );

    tracing::info!(
        message = "starting gRPC server",
        socket_address = %socket_address,
    );
    Ok(server.serve().await?)
}

const SERVICE_NAME: &'static str = "pipeline-ingress";

#[tokio::main]
async fn main() -> Result<(), ConfigurationError> {
    let _guard = setup_tracing(SERVICE_NAME)?;

    tracing::info!("starting up!");

    match handler().await {
        Ok(res) => {
            tracing::info!("shutting down");
            Ok(res)
        }
        Err(err) => {
            tracing::error!(
                message = "configuration error",
                error = ?err,
            );
            Err(err)
        }
    }
}
