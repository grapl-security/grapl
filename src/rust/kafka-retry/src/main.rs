use clap::Parser;
use futures::stream::StreamExt;
use kafka::{
    config::{
        ProducerConfig,
        RetryConsumerConfig,
    },
    ConfigurationError,
    RetryProcessor,
};
use opentelemetry::{
    global,
    sdk::propagation::TraceContextPropagator,
};
use tracing::instrument::WithSubscriber;
use tracing_subscriber::{
    prelude::*,
    EnvFilter,
};

#[derive(clap::Parser, Clone, Debug)]
struct Config {
    #[clap(flatten)]
    kafka_retry_consumer_config: RetryConsumerConfig,

    #[clap(flatten)]
    kafka_producer_config: ProducerConfig,

    #[clap(long, env = "KAFKA_RETRY_WORKER_POOL_SIZE")]
    kafka_retry_worker_pool_size: usize,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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

    Ok(handler().await?)
}

async fn handler() -> Result<(), ConfigurationError> {
    let config = Config::parse();
    let retry_processor = RetryProcessor::new(
        config.kafka_retry_consumer_config,
        config.kafka_producer_config,
    )?;

    retry_processor
        .stream()
        .for_each_concurrent(config.kafka_retry_worker_pool_size, |res| async move {
            if let Err(e) = res {
                tracing::error!(
                    message = "Error processing Kafka message",
                    reason =% e,
                );
            } else {
                // TODO: collect metrics
                tracing::debug!("Processed Kafka message");
            }
        })
        .with_current_subscriber()
        .await;

    Ok(())
}
