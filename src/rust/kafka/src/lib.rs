pub mod config;

use std::marker::PhantomData;

use config::{
    ConsumerConfig,
    ProducerConfig,
};
use futures::{
    stream::{
        Stream,
        StreamExt,
    },
    Future,
    FutureExt,
    TryFutureExt,
    TryStreamExt,
};
use rdkafka::{
    config::ClientConfig,
    consumer::stream_consumer::StreamConsumer,
    error::KafkaError,
    producer::{
        FutureProducer,
        FutureRecord,
    },
    util::Timeout,
    Message,
};
use rust_proto::{
    SerDe,
    SerDeError,
};
use thiserror::Error;

//
// Kafka configurations
//

#[non_exhaustive]
#[derive(Error, Debug)]
pub enum ConfigurationError {
    #[error("failed to construct kafka producer {0}")]
    ProducerCreateFailed(KafkaError),

    #[error("failed to construct kafka consumer {0}")]
    ConsumerCreateFailed(KafkaError),

    #[error("failed to subscribe kafka consumer {0}")]
    SubscriptionFailed(KafkaError),
}

fn configure(
    bootstrap_servers: String,
    sasl_username: String,
    sasl_password: String,
) -> ClientConfig {
    if bootstrap_servers.starts_with("SASL_SSL") {
        // running in aws w/ ccloud
        // these configuration values were recommended by confluent cloud:
        // https://docs.confluent.io/cloud/current/client-apps/config-client.html
        ClientConfig::new()
            .set("bootstrap.servers", bootstrap_servers)
            .set("security.protocol", "SASL_SSL")
            .set("sasl.mechanisms", "PLAIN")
            .set("sasl.username", sasl_username)
            .set("sasl.password", sasl_password)
            .set("broker.address.ttl", "30000")
            .set("api.version.request", "true")
            .set("api.version.fallback.ms", "0")
            .set("broker.version.fallback", "0.10.0.0")
            .to_owned()
    } else {
        // running locally
        ClientConfig::new()
            .set("bootstrap.servers", bootstrap_servers)
            .set("security.protocol", "PLAINTEXT")
            .to_owned()
    }
}

//
// Producer
//

fn producer(
    bootstrap_servers: String,
    sasl_username: String,
    sasl_password: String,
) -> Result<FutureProducer, ConfigurationError> {
    configure(bootstrap_servers, sasl_username, sasl_password)
        .set("acks", "all")
        .create()
        .map_err(|e| ConfigurationError::ProducerCreateFailed(e))
}

#[non_exhaustive]
#[derive(Error, Debug)]
pub enum ProducerError {
    #[error("failed to serialize message {0}")]
    SerializationError(#[from] SerDeError),

    #[error("failed to deliver message to kafka {0}")]
    KafkaError(#[from] KafkaError),
}

#[derive(Clone)]
pub struct Producer<T>
where
    T: SerDe,
{
    producer: FutureProducer,
    topic: String,
    _t: PhantomData<T>,
}

/// A producer publishes data to a topic. This producer serializes the data it
/// is given before publishing.
impl<T: SerDe> Producer<T> {
    pub fn new(config: ProducerConfig) -> Result<Self, ConfigurationError> {
        Ok(Self {
            producer: producer(
                config.bootstrap_servers,
                config.sasl_username,
                config.sasl_password,
            )?,
            topic: config.topic,
            _t: PhantomData,
        })
    }

    #[tracing::instrument(err, skip(self))]
    pub async fn send(&self, msg: T) -> Result<(), ProducerError> {
        let serialized = msg.serialize()?.to_vec();
        let record: FutureRecord<Vec<u8>, Vec<u8>> =
            FutureRecord::to(&self.topic).payload(&serialized);

        self.producer
            .send(record, Timeout::Never)
            .map(|res| -> Result<(), ProducerError> {
                res.map_err(|(e, _)| -> ProducerError { e.into() })
                    .map(|(partition, offset)| {
                        tracing::trace!(
                            message = "wrote message",
                            partition = partition,
                            offset = offset,
                        );
                    })
            })
            .await
    }
}

//
// Consumer
//

fn consumer(
    bootstrap_servers: String,
    sasl_username: String,
    sasl_password: String,
    consumer_group_name: String,
) -> Result<StreamConsumer, ConfigurationError> {
    configure(bootstrap_servers, sasl_username, sasl_password)
        .set("group.id", consumer_group_name)
        .set("enable.auto.commit", "true")
        .set("auto.offset.reset", "earliest")
        .set("session.timeout.ms", "45000")
        .create()
        .map_err(|e| ConfigurationError::ConsumerCreateFailed(e))
}

#[non_exhaustive]
#[derive(Error, Debug)]
pub enum ConsumerError {
    #[error("failed to deserialize message {0}")]
    DeserializationError(#[from] SerDeError),

    #[error("failed to consume message from kafka {0}")]
    KafkaConsumptionFailed(#[from] KafkaError),

    #[error("message payload absent")]
    PayloadAbsent,
}

/// A consumer consumes data from a topic. This consumer deserializes each
/// message after consuming it, and yields the deserialized message to the
/// caller.
pub struct Consumer<T>
where
    T: SerDe,
{
    consumer: StreamConsumer,
    topic: String,
    _t: PhantomData<T>,
}

impl<T: SerDe> Consumer<T> {
    pub fn new(config: ConsumerConfig) -> Result<Self, ConfigurationError> {
        Ok(Self {
            consumer: consumer(
                config.bootstrap_servers,
                config.sasl_username,
                config.sasl_password,
                config.consumer_group_name,
            )?,
            topic: config.topic,
            _t: PhantomData,
        })
    }

    #[tracing::instrument(err, skip(self))]
    pub fn stream(
        &self,
    ) -> Result<impl Stream<Item = Result<T, ConsumerError>> + '_, ConfigurationError> {
        // the .subscribe(..) call must be fully-qualified here because the
        // Consumer name is shadowed in this crate
        match rdkafka::consumer::Consumer::subscribe(&self.consumer, &[&self.topic]) {
            Ok(()) => Ok(self
                .consumer
                .stream()
                .map(|res| -> Result<T, ConsumerError> {
                    res.map_err(ConsumerError::from).and_then(|msg| {
                        T::deserialize(msg.payload().ok_or(ConsumerError::PayloadAbsent)?)
                            .map_err(ConsumerError::from)
                    })
                })),
            Err(e) => Err(ConfigurationError::SubscriptionFailed(e)),
        }
    }
}

//
// StreamProcessor
//

#[non_exhaustive]
#[derive(Error, Debug)]
pub enum StreamProcessorError {
    #[error("encountered consumer error {0}")]
    ConsumerError(#[from] ConsumerError),

    #[error("encountered producer error {0}")]
    ProducerError(#[from] ProducerError),

    #[error("encountered event handler error {0}")]
    EventHandlerError(String),
}

/// A stream processor consumes data from a topic, does things with the data,
/// and produces data to another topic. This stream processor deserializes the
/// data after consuming and serializes data before producing.
pub struct StreamProcessor<C, P>
where
    C: SerDe,
    P: SerDe,
{
    consumer: Consumer<C>,
    producer: Producer<P>,
}

impl<C, P> StreamProcessor<C, P>
where
    C: SerDe,
    P: SerDe,
{
    pub fn new(
        consumer_config: ConsumerConfig,
        producer_config: ProducerConfig,
    ) -> Result<StreamProcessor<C, P>, ConfigurationError> {
        Ok(StreamProcessor {
            consumer: Consumer::new(consumer_config)?,
            producer: Producer::new(producer_config)?,
        })
    }

    #[tracing::instrument(err, skip(self, event_handler))]
    pub fn stream<'a, F, R, E>(
        &'a self,
        event_handler: F,
    ) -> Result<impl Stream<Item = Result<(), StreamProcessorError>> + '_, ConfigurationError>
    where
        F: FnMut(Result<C, StreamProcessorError>) -> R + 'a,
        R: Future<Output = Result<Option<P>, E>> + 'a,
        E: Into<StreamProcessorError> + 'a,
    {
        Ok(self
            .consumer
            .stream()?
            .map_err(StreamProcessorError::from)
            .then(event_handler)
            .then(move |result| async move {
                match result {
                    Ok(msg) => match msg {
                        Some(msg) => {
                            self.producer
                                .clone()
                                .send(msg)
                                .map_err(StreamProcessorError::from)
                                .await
                        }
                        None => Ok(()),
                    },
                    Err(e) => Err(e.into()),
                }
            }))
    }
}
