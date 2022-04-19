use std::{
    fmt::Display,
    marker::PhantomData,
};

use futures::{
    stream::{
        Stream,
        StreamExt,
    },
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
use rust_proto_new::{
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
    pub fn new(
        bootstrap_servers: String,
        sasl_username: String,
        sasl_password: String,
        topic: String,
    ) -> Result<Producer<T>, ConfigurationError> {
        Ok(Producer {
            producer: producer(bootstrap_servers, sasl_username, sasl_password)?,
            topic,
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
    pub fn new(
        bootstrap_servers: String,
        sasl_username: String,
        sasl_password: String,
        consumer_group_name: String,
        topic: String,
    ) -> Result<Consumer<T>, ConfigurationError> {
        Ok(Consumer {
            consumer: consumer(
                bootstrap_servers,
                sasl_username,
                sasl_password,
                consumer_group_name,
            )?,
            topic,
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

impl<C: SerDe, P: SerDe> StreamProcessor<C, P> {
    pub fn new(
        bootstrap_servers: String,
        sasl_username: String,
        sasl_password: String,
        consumer_group_name: String,
        consumer_topic: String,
        producer_topic: String,
    ) -> Result<StreamProcessor<C, P>, ConfigurationError> {
        Ok(StreamProcessor {
            consumer: Consumer::new(
                bootstrap_servers.clone(),
                sasl_username.clone(),
                sasl_password.clone(),
                consumer_group_name,
                consumer_topic,
            )?,
            producer: Producer::new(
                bootstrap_servers,
                sasl_username,
                sasl_password,
                producer_topic,
            )?,
        })
    }

    #[tracing::instrument(err, skip(self, event_handler))]
    pub fn stream<'a, F, E>(
        &'a self,
        event_handler: F,
    ) -> Result<impl Stream<Item = Result<(), StreamProcessorError>> + 'a, ConfigurationError>
    where
        F: FnMut(Result<C, StreamProcessorError>) -> Result<Option<P>, E> + 'a,
        E: Display,
    {
        Ok(self
            .consumer
            .stream()?
            .map_err(StreamProcessorError::from)
            .map(event_handler)
            .map_err(|e| StreamProcessorError::EventHandlerError(e.to_string()))
            .and_then(move |msg| async move {
                match msg {
                    Some(msg) => {
                        // The underlying FutureProducer::clone() call is inexpensive,
                        // so I think it's acceptable here.
                        self.producer
                            .clone()
                            .send(msg)
                            .map_err(StreamProcessorError::from)
                            .await
                    }
                    None => Ok(()),
                }
            }))
    }
}
