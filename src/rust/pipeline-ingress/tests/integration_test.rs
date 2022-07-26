#![cfg(feature = "integration_tests")]

use std::time::Duration;

use bytes::Bytes;
use clap::Parser;
use futures::StreamExt;
use grapl_tracing::{
    setup_tracing,
    WorkerGuard,
};
use kafka::{
    config::ConsumerConfig,
    Consumer,
    ConsumerError,
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
        pipeline::{
            v1beta1::RawLog,
            v1beta2::Envelope,
        },
    },
};
use test_context::{
    test_context,
    AsyncTestContext,
};
use tokio::sync::oneshot;
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
async fn test_publish_raw_log_sends_message_to_kafka(ctx: &mut PipelineIngressTestContext) {
    let event_source_id = Uuid::new_v4();
    let tenant_id = Uuid::new_v4();
    let log_event: Bytes = "test".into();

    tracing::info!("configuring kafka consumer");

    let kafka_consumer =
        Consumer::new(ctx.consumer_config.clone()).expect("could not configure kafka consumer");

    // we'll use this channel to communicate that the consumer is ready to
    // consume messages
    let (tx, rx) = oneshot::channel::<()>();

    tracing::info!("creating kafka subscriber thread");
    let kafka_subscriber = tokio::task::spawn(async move {
        let stream = kafka_consumer.stream();

        // notify the consumer that we're ready to receive messages
        tx.send(())
            .expect("failed to notify sender that consumer is consuming");

        let contains_expected =
            stream.any(|res: Result<Envelope<RawLog>, ConsumerError>| async move {
                let envelope = res.expect("error consuming message from kafka");
                let metadata = envelope.metadata;
                let raw_log = envelope.inner_message;
                let expected_log_event: Bytes = "test".into();

                tracing::debug!(message = "consumed kafka message");

                metadata.tenant_id == tenant_id
                    && metadata.event_source_id == event_source_id
                    && raw_log.log_event == expected_log_event
            });

        tracing::info!("consuming kafka messages for 30s");
        assert!(
            tokio::time::timeout(Duration::from_millis(30000), contains_expected)
                .await
                .expect("failed to consume expected message within 30s")
        );
    });

    // wait for the kafka consumer to start consuming
    tracing::info!("waiting for kafka consumer to report ready");
    rx.await
        .expect("failed to receive notification that consumer is consuming");

    tracing::info!("sending publish_raw_log request");
    ctx.grpc_client
        .publish_raw_log(PublishRawLogRequest {
            event_source_id,
            tenant_id,
            log_event,
        })
        .await
        .expect("received error response");

    tracing::info!("waiting for kafka_subscriber to complete");
    kafka_subscriber
        .instrument(tracing::debug_span!("kafka_subscriber"))
        .await
        .expect("could not join kafka subscriber");
}
