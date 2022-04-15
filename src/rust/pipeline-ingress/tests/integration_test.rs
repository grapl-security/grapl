#![cfg(feature = "new_integration_tests")]

use std::time::{
    Duration,
    SystemTime,
};

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
use tracing_subscriber::{
    prelude::*,
    EnvFilter,
};
use uuid::Uuid;

struct PipelineIngressTestContext {
    grpc_client: PipelineIngressClient,
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
            .unwrap();

        // register a subscriber
        let filter = EnvFilter::from_default_env();
        tracing_subscriber::registry()
            .with(filter)
            .with(log_layer)
            .with(tracing_opentelemetry::layer().with_tracer(tracer))
            .init();

        tracing::info!("logger configured successfully");

        let endpoint = format!(
            "http://{}",
            std::env::var("PIPELINE_INGRESS_BIND_ADDRESS")
                .expect("missing environment variable PIPELINE_INGRESS_BIND_ADDRESS")
        );

        tracing::info!(
            "waiting 10s for pipeline-ingress to report healthy at: {}",
            endpoint
        );

        HealthcheckClient::wait_until_healthy(
            endpoint.clone(),
            "pipeline-ingress",
            Duration::from_millis(10000),
            Duration::from_millis(500),
        )
        .await
        .expect("pipeline-ingress never reported healthy");

        let grpc_client = PipelineIngressClient::connect(endpoint.clone())
            .await
            .expect("could not configure gRPC client");

        PipelineIngressTestContext { grpc_client }
    }
}

#[test_context(PipelineIngressTestContext)]
#[tokio::test]
async fn test_publish_raw_log_sends_message_to_kafka(ctx: &mut PipelineIngressTestContext) {
    let event_source_id = Uuid::new_v4();
    let tenant_id = Uuid::new_v4();

    let kafka_subscriber = tokio::task::spawn(async move {
        let kafka_consumer = Consumer::new("raw-logs").expect("could not configure kafka consumer");
        let contains_expected = kafka_consumer
            .stream()
            .expect("could not subscribe to the raw-logs topic")
            .any(|res: Result<Envelope<RawLog>, ConsumerError>| async move {
                let metadata = res.expect("error consuming message from kafka").metadata;
                metadata.tenant_id == tenant_id && metadata.event_source_id == event_source_id
            });

        assert!(
            tokio::time::timeout(Duration::from_millis(5000), contains_expected,)
                .await
                .expect("failed to consume expected message within 5s")
        );
    });

    let res = ctx
        .grpc_client
        .publish_raw_log(PublishRawLogRequest {
            event_source_id,
            tenant_id,
            log_event: "test".into(),
        })
        .await
        .expect("received error response");

    assert!(res.created_time < SystemTime::now());

    kafka_subscriber
        .await
        .expect("could not join kafka subscriber");
}
