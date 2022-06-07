use std::time::Duration;

use clap::Parser;
use kafka::config::ConsumerConfig;
use opentelemetry::{
    global,
    sdk::propagation::TraceContextPropagator,
};
use rust_proto_new::{
    graplinc::grapl::api::pipeline_ingress::v1beta1::client::PipelineIngressClient,
    protocol::healthcheck::client::HealthcheckClient,
};
use test_context::AsyncTestContext;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{
    prelude::*,
    EnvFilter,
};

pub struct GeneratorDispatcherTestContext {
    pub pipeline_ingress_client: PipelineIngressClient,
    pub consumer_config: ConsumerConfig,
    pub _guard: WorkerGuard,
}

static CONSUMER_TOPIC: &'static str = "raw-logs";

#[async_trait::async_trait]
impl AsyncTestContext for GeneratorDispatcherTestContext {
    async fn setup() -> Self {
        let (non_blocking, _guard) = tracing_appender::non_blocking(std::io::stdout());

        // initialize json logging layer
        let log_layer = tracing_subscriber::fmt::layer()
            .json()
            .with_writer(non_blocking);

        // initialize tracing layer
        global::set_text_map_propagator(TraceContextPropagator::new());
        let tracer = opentelemetry_jaeger::new_pipeline()
            .with_service_name("generator-dispatcher-integration-tests")
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
        let pipeline_ingress_client = PipelineIngressClient::connect(endpoint.clone())
            .await
            .expect("could not configure gRPC client");

        let consumer_config = ConsumerConfig {
            topic: CONSUMER_TOPIC.to_string(),
            ..ConsumerConfig::parse()
        };
        GeneratorDispatcherTestContext {
            pipeline_ingress_client,
            consumer_config,
            _guard,
        }
    }
}
