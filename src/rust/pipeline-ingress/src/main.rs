use std::{
    env::VarError,
    num::ParseIntError,
    time::SystemTime,
};

use kafka::{
    ConfigurationError as KafkaConfigurationError,
    Producer,
    ProducerError,
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
}

// async fn check_server_running(
//     socket_address: String
// ) -> Result<HealthcheckStatus, HealthcheckError> {
//     //let stream = TcpStream::connect(socket_address.parse());
//     Ok(HealthcheckStatus::Serving) // FIXME
// }

// async fn check_kafka() -> Result<HealthcheckStatus, HealthcheckError> {
//     Ok(HealthcheckStatus::Serving) // FIXME
// }

#[tokio::main]
async fn main() -> Result<(), ConfigurationError> {
    let socket_address = std::env::var("PIPELINE_INGRESS_BIND_ADDRESS")?;
    let healthcheck_polling_interval_ms =
        std::env::var("PIPELINE_INGRESS_HEALTHCHECK_POLLING_INTERVAL_MS")?.parse()?;

    let producer: Producer<Envelope<RawLog>> = Producer::new("raw-logs")?;

    let (server, _) = PipelineIngressServer::new(
        IngressApi::new(producer),
        TcpListener::bind(socket_address).await?,
        || async { Ok(HealthcheckStatus::Serving) },
        healthcheck_polling_interval_ms,
    );

    Ok(server.serve().await?)
}
