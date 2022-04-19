#![cfg(feature = "new_integration_tests")]

use std::time::Duration;

use bytes::Bytes;
use futures::StreamExt;
use kafka::{
    Consumer,
    ConsumerError,
};
use opentelemetry::{
    global,
    sdk::propagation::TraceContextPropagator,
};
use rust_proto_new::graplinc::grapl::{
    api::pipeline_ingress::v1beta1::{
        client::{
            HealthcheckClient,
            PipelineIngressClient,
        },
        PublishRawLogRequest,
    },
    pipeline::{
        v1beta1::RawLog,
        v1beta2::Envelope,
    },
};
use test_context::{
    test_context,
    AsyncTestContext,
};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{
    prelude::*,
    EnvFilter,
};
use uuid::Uuid;

struct PipelineIngressTestContext {
    grpc_client: PipelineIngressClient,
    bootstrap_servers: String,
    sasl_username: String,
    sasl_password: String,
    consumer_group_name: String,
    _guard: WorkerGuard,
}

#[async_trait::async_trait]
impl AsyncTestContext for PipelineIngressTestContext {
    async fn setup() -> Self {
        let (non_blocking, _guard) = tracing_appender::non_blocking(std::io::stdout());

        // initialize json logging layer
        let log_layer = tracing_subscriber::fmt::layer()
            .json()
            .with_writer(non_blocking);

        // initialize tracing layer
        global::set_text_map_propagator(TraceContextPropagator::new());
        let tracer = opentelemetry_jaeger::new_pipeline()
            .with_service_name("pipeline-ingress-integration-tests")
            .install_batch(opentelemetry::runtime::Tokio)
            .expect("could not configure tracer");

        // register a subscriber
        let filter = EnvFilter::from_default_env();
        tracing_subscriber::registry()
            .with(filter)
            .with(log_layer)
            .with(tracing_opentelemetry::layer().with_tracer(tracer))
            .init();

        tracing::info!("logger configured successfully");

        let endpoint = std::env::var("PIPELINE_INGRESS_CLIENT_ADDRESS")
            .expect("missing environment variable PIPELINE_INGRESS_CLIENT_ADDRESS");

        let bootstrap_servers = std::env::var("KAFKA_BOOTSTRAP_SERVERS")
            .expect("missing environment variable KAFKA_BOOTSTRAP_SERVERS");
        let sasl_username = std::env::var("PIPELINE_INGRESS_KAFKA_SASL_USERNAME")
            .expect("missing environment variable PIPELINE_INGRESS_KAFKA_SASL_USERNAME");
        let sasl_password = std::env::var("PIPELINE_INGRESS_KAFKA_SASL_PASSWORD")
            .expect("missing environment variable PIPELINE_INGRESS_KAFKA_SASL_PASSWORD");
        let consumer_group_name = std::env::var("PIPELINE_INGRESS_TEST_KAFKA_CONSUMER_GROUP_NAME")
            .expect("missing environment variable PIPELINE_INGRESS_TEST_KAFKA_CONSUMER_GROUP_NAME");

        tracing::info!(
            message = "waiting 10s for pipeline-ingress to report healthy",
            endpoint = %endpoint,
        );

        HealthcheckClient::wait_until_healthy(
            endpoint.clone(),
            "graplinc.grapl.api.pipeline_ingress.v1beta1.PipelineIngressService",
            Duration::from_millis(10000),
            Duration::from_millis(500),
        )
        .await
        .expect("pipeline-ingress never reported healthy");

        tracing::info!("connecting pipeline-ingress gRPC client");
        let grpc_client = PipelineIngressClient::connect(endpoint.clone())
            .await
            .expect("could not configure gRPC client");

        PipelineIngressTestContext {
            grpc_client,
            bootstrap_servers,
            sasl_username,
            sasl_password,
            consumer_group_name,
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

    let bootstrap_servers = ctx.bootstrap_servers.clone();
    let sasl_username = ctx.sasl_username.clone();
    let sasl_password = ctx.sasl_password.clone();
    let consumer_group_name = ctx.consumer_group_name.clone();

    tracing::info!("configuring kafka consumer");
    let kafka_consumer = Consumer::new(
        bootstrap_servers,
        sasl_username,
        sasl_password,
        consumer_group_name,
        "raw-logs".to_string(),
    )
    .expect("could not configure kafka consumer");

    tracing::info!("creating kafka subscriber thread");
    let kafka_subscriber = tokio::task::spawn(async move {
        let contains_expected = kafka_consumer
            .stream()
            .expect("could not subscribe to the raw-logs topic")
            .any(|res: Result<Envelope<RawLog>, ConsumerError>| async move {
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

    // we throw in a healthy sleep here to make well sure the kafka consumer is
    // actually consuming before we throw a message at it
    tokio::time::sleep(Duration::from_millis(5000)).await;

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
        .await
        .expect("could not join kafka subscriber");
}
