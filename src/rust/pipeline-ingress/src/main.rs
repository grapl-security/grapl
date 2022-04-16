use std::{
    env::VarError,
    num::ParseIntError,
    time::{
        Duration,
        SystemTime,
    },
};

use kafka::{
    ConfigurationError as KafkaConfigurationError,
    Producer,
    ProducerError,
};
use opentelemetry::{
    global,
    sdk::propagation::TraceContextPropagator,
    trace::TraceError,
};
use rust_proto_new::graplinc::grapl::{
    api::pipeline_ingress::v1beta1::{
        server::{
            ConfigurationError as ServerConfigurationError,
            PipelineIngressApi,
            PipelineIngressServer,
        },
        HealthcheckStatus,
        PublishRawLogRequest,
        PublishRawLogResponse,
    },
    pipeline::{
        v1beta1::{
            Metadata,
            RawLog,
        },
        v1beta2::Envelope,
    },
};
use thiserror::Error;
use tokio::net::TcpListener;
use tracing_subscriber::{
    prelude::*,
    EnvFilter,
};
use uuid::Uuid;

#[non_exhaustive]
#[derive(Debug, Error)]
enum IngressApiError {
    #[error("failed to send message to kafka {0}")]
    ProducerError(#[from] ProducerError),
}

struct IngressApi {
    producer: Producer<Envelope<RawLog>>,
}

impl IngressApi {
    fn new(producer: Producer<Envelope<RawLog>>) -> Self {
        IngressApi { producer }
    }
}

#[async_trait::async_trait]
impl PipelineIngressApi<IngressApiError> for IngressApi {
    async fn publish_raw_log(
        &self,
        request: PublishRawLogRequest,
    ) -> Result<PublishRawLogResponse, IngressApiError> {
        let created_time = SystemTime::now();
        let last_updated_time = created_time;
        let tenant_id = request.tenant_id;
        let event_source_id = request.event_source_id;
        // TODO: trace_id should be generated at the edge. This service is
        // currently "the edge" but that won't be true forever. When there is an
        // actual edge service, that service should be responsible for
        // generating the trace_id.
        let trace_id = Uuid::new_v4();
        self.producer
            .send(Envelope::new(
                Metadata::new(
                    tenant_id,
                    trace_id,
                    0,
                    created_time,
                    last_updated_time,
                    event_source_id,
                ),
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

    #[error("failed to configure gRPC server {0}")]
    Server(#[from] ServerConfigurationError),

    #[error("missing environment variable {0}")]
    EnvironmentVariable(#[from] VarError),

    #[error("failed to bind socket address {0}")]
    SocketError(#[from] std::io::Error),

    #[error("failed to parse integer value {0}")]
    ParseInt(#[from] ParseIntError),

    #[error("failed to configure tracing {0}")]
    Tracing(#[from] TraceError),
}

#[tracing::instrument(err)]
async fn handler() -> Result<(), ConfigurationError> {
    let socket_address = std::env::var("PIPELINE_INGRESS_BIND_ADDRESS")?;

    let healthcheck_polling_interval_ms =
        std::env::var("PIPELINE_INGRESS_HEALTHCHECK_POLLING_INTERVAL_MS")?.parse()?;

    let bootstrap_servers = std::env::var("KAFKA_BOOTSTRAP_SERVERS")?;
    let sasl_username = std::env::var("KAFKA_SASL_USERNAME")?;
    let sasl_password = std::env::var("KAFKA_SASL_PASSWORD")?;

    let raw_logs_topic = "raw-logs".to_string();

    tracing::info!(
        message = "configuring kafka producer",
        bootstrap_servers = %bootstrap_servers,
        topic = %raw_logs_topic,
    );
    let producer: Producer<Envelope<RawLog>> = Producer::new(
        bootstrap_servers.clone(),
        sasl_username,
        sasl_password,
        raw_logs_topic.clone(),
    )?;
    tracing::info!(
        message = "kafka producer configured successfully",
        bootstrap_servers = %bootstrap_servers,
        topic = %raw_logs_topic,
    );

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
        socket_address = %socket_address
    );

    tracing::info!(
        message = "starting gRPC server",
        socket_address = %socket_address,
    );
    Ok(server.serve().await?)
}

#[tokio::main]
async fn main() -> Result<(), ConfigurationError> {
    let (non_blocking, _guard) = tracing_appender::non_blocking(std::io::stdout());

    // initialize json logging layer
    let log_layer = tracing_subscriber::fmt::layer()
        .json()
        .with_writer(non_blocking);

    // initialize tracing layer
    global::set_text_map_propagator(TraceContextPropagator::new());
    let tracer = opentelemetry_jaeger::new_pipeline()
        .with_service_name("pipeline-ingress")
        .install_batch(opentelemetry::runtime::Tokio)?;

    // register a subscriber
    let filter = EnvFilter::from_default_env();
    tracing_subscriber::registry()
        .with(filter)
        .with(log_layer)
        .with(tracing_opentelemetry::layer().with_tracer(tracer))
        .init();

    tracing::info!("logger configured successfully");
    tracing::info!("starting up!");

    match handler().await {
        Ok(res) => {
            tracing::info!("shutting down");
            Ok(res)
        }
        Err(err) => {
            tracing::error!("configuration error: {}", err);
            Err(err)
        }
    }
}
