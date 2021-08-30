use std::time::Duration;

use grapl_config::env_helpers::FromEnv;
use rdkafka::{
    config::FromClientConfig,
    error::KafkaError,
    producer::{
        FutureProducer,
        FutureRecord,
    },
};

const KAFKA_PRODUCER_TOPIC: &str = "KAFKA_PRODUCER_TOPIC";
const KAFKA_PRODUCER_BROKERS: &str = "KAFKA_PRODUCER_BROKERS";
const KAFKA_PRODUCER_CLIENT_ID: &str = "KAFKA_PRODUCER_CLIENT_ID";
const KAFKA_PRODUCER_BUFFERING_MAX_MS: &str = "KAFKA_PRODUCER_BUFFERING_MAX_MS";
const KAFKA_PRODUCER_LINGER_MS: &str = "KAFKA_PRODUCER_LINGER_MS";

#[derive(Debug, thiserror::Error)]
pub enum EventProducerError {
    #[error("KafkaError")]
    KafkaError(#[from] KafkaError),
}

#[derive(Clone)]
pub struct KafkaProducer {
    producer: FutureProducer,
    pub(crate) topic_name: String,
}

impl KafkaProducer {
    pub async fn produce(&self, payload: &[u8]) -> Result<(), EventProducerError> {
        let record: FutureRecord<[u8], _> = FutureRecord {
            topic: &self.topic_name,
            payload: Some(payload),
            partition: None,
            key: None,
            timestamp: None,
            headers: None,
        };

        match self.producer.send(record, Duration::from_secs(3)).await {
            Ok((partition, offset)) => {
                tracing::debug!(
                    message="Event published",
                    topic=%&self.topic_name,
                    partition=%partition,
                    offset=%offset,
                );
            }
            Err((e, _)) => {
                tracing::error!(
                    message="Failed to send message to kafka",
                    topic=%&self.topic_name,
                    error=%e,
                );
                return Err(e.into());
            }
        }

        Ok(())
    }
}

// We may want to add more tunables https://github.com/edenhill/librdkafka/blob/master/INTRODUCTION.md#performance
impl FromEnv<Self> for KafkaProducer {
    fn from_env() -> Self {
        let topic_name = std::env::var(KAFKA_PRODUCER_TOPIC).expect(KAFKA_PRODUCER_TOPIC);
        let brokers = std::env::var(KAFKA_PRODUCER_BROKERS).expect(KAFKA_PRODUCER_BROKERS);
        let producer_client_id =
            std::env::var(KAFKA_PRODUCER_CLIENT_ID).expect(KAFKA_PRODUCER_CLIENT_ID);

        let mut client_config = rdkafka::ClientConfig::new();
        client_config
            .set("client.id", &producer_client_id)
            .set("bootstrap.servers", brokers);

        if let Ok(queue_buffering_max_ms) = std::env::var(KAFKA_PRODUCER_BUFFERING_MAX_MS) {
            client_config.set("queue.buffering.max.ms", queue_buffering_max_ms);
        }

        if let Ok(linger_ms) = std::env::var(KAFKA_PRODUCER_LINGER_MS) {
            client_config.set("linger.ms", linger_ms);
        }

        let producer =
            FutureProducer::from_config(&client_config).expect("FutureProducer::from_config");

        Self {
            producer,
            topic_name,
        }
    }
}
