#![cfg(feature = "integration_tests")]

use bytes::Bytes;
use clap::Parser;
use grapl_tracing::{
    setup_tracing,
    WorkerGuard,
};
use kafka::{
    config::ConsumerConfig,
    test_utils::topic_scanner::KafkaTopicScanner,
};
use rust_proto::{
    client_factory::{
        build_grpc_client_with_options,
        services::PipelineIngressClientConfig,
        BuildGrpcClientOptions,
    },
    graplinc::grapl::{
        api::pipeline_ingress::v1beta1::{
            client::PipelineIngressClient,
            PublishRawLogRequest,
        },
        pipeline::v1beta1::{
            Envelope,
            RawLog,
        },
    },
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
        let pipeline_ingress_client = build_grpc_client_with_options(
            client_config,
            BuildGrpcClientOptions {
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
        .contains(move |envelope: Envelope<RawLog>| -> bool {
            let envelope_tenant_id = envelope.tenant_id();
            let envelope_event_source_id = envelope.event_source_id();
            let raw_log = envelope.inner_message();
            let expected_log_event: Bytes = "test".into();

            tracing::debug!(message = "consumed kafka message");

            envelope_tenant_id == tenant_id
                && envelope_event_source_id == event_source_id
                && raw_log.log_event() == expected_log_event
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
