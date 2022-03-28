use std::{net::SocketAddr, time::SystemTime};

use async_trait::async_trait;

use rust_proto_new::graplinc::grapl::{
    api::pipeline_ingress::v1beta1::{
        PublishRawLogsRequest,
        PublishRawLogsResponse,
        server::{
            PipelineIngressApi,
            PipelineIngressServer,
            ConfigurationError as ServerConfigurationError,
        }
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

#[derive(Error)]
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
    async fn publish_raw_logs(
        &self, request: PublishRawLogsRequest
    ) -> Result<PublishRawLogsResponse, IngressApiError> {
        let created_time = SystemTime::now();
        let last_updated_time = created_time;
        let tenant_id = request.tenant_id;
        let event_source_id = request.event_source_id;
        let trace_id = event_source_id; // FIXME!!!
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
                RawLog::new(log_event),
            )
        ).await?
    }
}

#[derive(Error)]
enum ConfigurationError {
    #[error("failed to configure kafka client {0}")]
    KafkaConfigurationError(#[from] KafkaConfigurationError),

    #[error("failed to configure gRPC server {0}")]
    ServerConfigurationError(#[from] ServerConfigurationError),
}

#[tokio::main]
fn main() -> Result<(), ConfigurationError> {
    let producer: Producer<Envelope<RawLog>> = Producer::new("raw-logs")?;
    let server = PipelineIngressServer::new(
        IngressApi::new(producer),
        "127.0.0.1:666".parse() // FIXME
    );

    server.serve()?;
}
