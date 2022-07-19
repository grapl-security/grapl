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
use tracing::instrument::WithSubscriber;

#[derive(clap::Parser, Clone, Debug)]
struct Config {
    #[clap(long, env = "KAFKA_RETRY_WORKER_POOL_SIZE")]
    kafka_retry_worker_pool_size: usize,
}

struct KafkaRetryConfig {
    kafka_retry_consumer_config: RetryConsumerConfig,
    kafka_producer_config: ProducerConfig,
    config: Config,
}

impl KafkaRetryConfig {
    fn parse() -> Self {
        KafkaRetryConfig {
            kafka_retry_consumer_config: RetryConsumerConfig::parse(),
            kafka_producer_config: ProducerConfig::parse(),
            config: Config::parse(),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = grapl_tracing::setup_tracing("kafka-retry")?;

    tracing::info!("logger configured successfully");
    tracing::info!("starting up!");

    Ok(handler().await?)
}

async fn handler() -> Result<(), ConfigurationError> {
    let config = KafkaRetryConfig::parse();
    let retry_processor = RetryProcessor::new(
        config.kafka_retry_consumer_config,
        config.kafka_producer_config,
    )?;

    retry_processor
        .stream()
        .for_each_concurrent(
            config.config.kafka_retry_worker_pool_size,
            |res| async move {
                if let Err(e) = res {
                    tracing::error!(
                        message = "Error processing Kafka message",
                        reason =% e,
                    );
                } else {
                    // TODO: collect metrics
                    tracing::debug!("Processed Kafka message");
                }
            },
        )
        .with_current_subscriber()
        .await;

    Ok(())
}
