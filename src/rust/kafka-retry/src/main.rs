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

struct KafkaRetryConfig {
    kafka_retry_consumer_config: RetryConsumerConfig,
    kafka_producer_config: ProducerConfig,
}

impl KafkaRetryConfig {
    fn parse() -> Self {
        KafkaRetryConfig {
            kafka_retry_consumer_config: RetryConsumerConfig::parse(),
            kafka_producer_config: ProducerConfig::parse(),
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

#[tracing::instrument(err)]
async fn handler() -> Result<(), ConfigurationError> {
    let config = KafkaRetryConfig::parse();
    let retry_processor = RetryProcessor::new(
        config.kafka_retry_consumer_config,
        config.kafka_producer_config,
    )?;

    retry_processor
        .stream()
        .for_each(|res| async move {
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
