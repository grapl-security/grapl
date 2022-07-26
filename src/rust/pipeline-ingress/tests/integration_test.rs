#![cfg(feature = "integration_tests")]

use bytes::Bytes;
use clap::Parser;
use grapl_tracing::{
    setup_tracing,
    WorkerGuard,
};
use kafka::{
    config::ConsumerConfig,
    test_utils::topic_contains::KafkaTopicScanner,
};
use rust_proto::graplinc::grapl::{
    api::pipeline_ingress::v1beta1::{
        client::PipelineIngressClient,
        PublishRawLogRequest,
    },
    pipeline::{
        v1beta1::RawLog,
        v1beta2::Envelope,
    },
};
use rust_proto_clients::{
    get_grpc_client_with_options,
    services::PipelineIngressClientConfig,
    GetGrpcClientOptions,
};
use test_context::{
    test_context,
    AsyncTestContext,
};
use tracing::Instrument;
use uuid::Uuid;

static CONSUMER_TOPIC: &'static str = "raw-logs";

struct PipelineIngressTestContext {
    grpc_client: PipelineIngressClient,
    consumer_config: ConsumerConfig,
    _guard: WorkerGuard,
}

#[async_trait::async_trait]
impl AsyncTestContext for PipelineIngressTestContext {
    async fn setup() -> Self {
        let _guard = setup_tracing("pipeline-ingress-integration-tests").expect("setup_tracing");

        let client_config = PipelineIngressClientConfig::parse();
        let pipeline_ingress_client = get_grpc_client_with_options(
            client_config,
            GetGrpcClientOptions {
                perform_healthcheck: true,
                ..Default::default()
            },
        )
        .await
        .expect("pipeline_ingress_client");
        let consumer_config = ConsumerConfig::with_topic(CONSUMER_TOPIC);

        PipelineIngressTestContext {
            grpc_client: pipeline_ingress_client,
            consumer_config,
            _guard,
        }
    }
}

#[test_context(PipelineIngressTestContext)]
#[tokio::test]
async fn test_publish_raw_log_sends_message_to_kafka(
    ctx: &mut PipelineIngressTestContext,
) -> Result<(), Box<dyn std::error::Error>> {
    let event_source_id = Uuid::new_v4();
    let tenant_id = Uuid::new_v4();
    let log_event: Bytes = "test".into();

    let kafka_scanner = KafkaTopicScanner::new(ctx.consumer_config.clone())?
        .contains(move |envelope: &Envelope<RawLog>| -> bool {
            let metadata = &envelope.metadata;
            let raw_log = &envelope.inner_message;
            let expected_log_event: Bytes = "test".into();

            tracing::debug!(message = "consumed kafka message");

            metadata.tenant_id == tenant_id
                && metadata.event_source_id == event_source_id
                && raw_log.log_event == expected_log_event
        })
        .await?;

    tracing::info!("sending publish_raw_log request");
    ctx.grpc_client
        .publish_raw_log(PublishRawLogRequest {
            event_source_id,
            tenant_id,
            log_event,
        })
        .await
        .expect("received error response");

    tracing::info!("waiting for kafka_scanner to complete");
    kafka_scanner
        .get_listen_result()
        .instrument(tracing::debug_span!("kafka_scanner"))
        .await??;
    Ok(())
}
