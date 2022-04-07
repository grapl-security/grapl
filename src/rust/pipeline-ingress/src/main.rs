use std::{time::SystemTime, net::AddrParseError, env::VarError};

use rust_proto_new::graplinc::grapl::{
    api::pipeline_ingress::v1beta1::{
        PublishRawLogRequest,
        PublishRawLogResponse,
        server::{
            PipelineIngressApi,
            PipelineIngressServer,
            ConfigurationError as ServerConfigurationError,
        }, HealthcheckStatus
    },
    pipeline::{
        v1beta1::{
            Metadata,
            RawLog,
        },
        v1beta2::Envelope,
    },
};

use kafka::{
    ConfigurationError as KafkaConfigurationError,
    Producer,
    ProducerError
};
use thiserror::Error;
use uuid::Uuid;

#[non_exhaustive]
#[derive(Debug, Error)]
enum IngressApiError {
    #[error("failed to send message to kafka {0}")]
    ProducerError(#[from] ProducerError),
}

struct IngressApi {
    producer: Producer<Envelope<RawLog>>
}

impl IngressApi {
    fn new(producer: Producer<Envelope<RawLog>>) -> Self {
        IngressApi {
            producer
        }
    }
}

#[async_trait::async_trait]
impl PipelineIngressApi<IngressApiError> for IngressApi {
    async fn publish_raw_log(
        &self, request: PublishRawLogRequest
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
        self.producer.send(
            Envelope::new(
                Metadata::new(
                    tenant_id,
                    trace_id,
                    0,
                    created_time,
                    last_updated_time,
                    event_source_id
                ),
                RawLog::new(request.log_event),
            )
        ).await?;

        Ok(PublishRawLogResponse::ok())
    }
}

#[non_exhaustive]
#[derive(Debug, Error)]
enum ConfigurationError {
    #[error("failed to configure kafka client {0}")]
    KafkaConfigurationError(#[from] KafkaConfigurationError),

    #[error("failed to configure gRPC server {0}")]
    ServerConfigurationError(#[from] ServerConfigurationError),

    #[error("failed to parse socket address {0}")]
    AddrParseError(#[from] AddrParseError),

    #[error("missing environment variable {0}")]
    MissingEnvironmentVariable(#[from] VarError),
}

#[tokio::main]
async fn main() -> Result<(), ConfigurationError> {
    let addr = std::env::var("PIPELINE_INGRESS_ENDPOINT_ADDRESS")?;
    let producer: Producer<Envelope<RawLog>> = Producer::new("raw-logs")?;
    let (server, _) = PipelineIngressServer::new(
        IngressApi::new(producer),
        addr.parse()?,
        || async { Ok(HealthcheckStatus::Serving) }, // FIXME
        50 // FIXME
    );

    Ok(server.serve().await?)
}
