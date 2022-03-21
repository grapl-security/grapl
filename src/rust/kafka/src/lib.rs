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

#[derive(Error, Debug)]
pub enum ConfigurationError {
    #[error("failed to retrieve value from environment variable {0}")]
    EnvironmentError(#[from] std::env::VarError),

    #[error("failed to construct kafka producer {0}")]
    ProducerCreateFailed(KafkaError),

    #[error("failed to construct kafka consumer {0}")]
    ConsumerCreateFailed(KafkaError),

    #[error("failed to subscribe kafka consumer {0}")]
    SubscriptionFailed(KafkaError),
}

fn configure() -> Result<ClientConfig, std::env::VarError> {
    let bootstrap_servers = std::env::var("KAFKA_BOOTSTRAP_SERVERS")?;
    if bootstrap_servers.starts_with("SASL_SSL") {
        // running in aws w/ ccloud
        // https://docs.confluent.io/cloud/current/client-apps/config-client.html#librdkafka-based-c-clients
        Ok(ClientConfig::new()
            .set("bootstrap.servers", bootstrap_servers)
            .set("security.protocol", "SASL_SSL")
            .set("sasl.mechanisms", "PLAIN")
            .set("sasl.username", std::env::var("KAFKA_SASL_USERNAME")?)
            .set("sasl.password", std::env::var("KAFKA_SASL_PASSWORD")?)
            .set("broker.address.ttl", "30000")
            .set("api.version.request", "true")
            .set("api.version.fallback.ms", "0")
            .set("broker.version.fallback", "0.10.0.0")
            .to_owned())
    } else {
        // running locally
        Ok(ClientConfig::new()
            .set("bootstrap.servers", bootstrap_servers)
            .set("security.protocol", "PLAINTEXT")
            .to_owned())
    }
}

//
// Producer
//

fn producer() -> Result<FutureProducer, ConfigurationError> {
    configure()?
        .set("acks", "all")
        .create()
        .map_err(|e| ConfigurationError::ProducerCreateFailed(e))
}

#[derive(Error, Debug)]
pub enum ProducerError {
    #[error("failed to serialize message {0}")]
    SerializationError(#[from] SerDeError),

    #[error("failed to deliver message to kafka {0}")]
    KafkaError(#[from] KafkaError),
}

pub struct Producer<T>
where
    T: SerDe,
{
    producer: FutureProducer,
    topic: String,
    _t: PhantomData<T>,
}

impl<T: SerDe> Clone for Producer<T> {
    fn clone(&self) -> Self {
        // this should be approximately just as cheap as
        // FutureProducer::clone(), which is documented to be very inexpensive,
        // so it's probably good enough?
        Producer {
            producer: self.producer.clone(),
            topic: self.topic.clone(),
            _t: PhantomData,
        }
    }
}

// A producer publishes data to a topic. This producer serializes the data it is
// given before publishing.
impl<T: SerDe> Producer<T> {
    pub fn new(topic: &str) -> Result<Producer<T>, ConfigurationError> {
        Ok(Producer {
            producer: producer()?,
            topic: topic.to_owned(),
            _t: PhantomData,
        })
    }

    pub async fn send(&self, msg: T) -> Result<(), ProducerError> {
        let serialized = msg.serialize()?.to_vec();
        let record: FutureRecord<Vec<u8>, Vec<u8>> =
            FutureRecord::to(&self.topic).payload(&serialized);

        self.producer
            .send(record, Timeout::Never)
            .map(|res| -> Result<(), ProducerError> {
                res.map_err(|(e, _)| -> ProducerError {
                    e.into() // TODO: add erroneous message to error
                })
                .map(|_| ()) // TODO: debug log partition and offset
            })
            .await
    }
}

//
// Consumer
//

fn consumer() -> Result<StreamConsumer, ConfigurationError> {
    configure()?
        .set("group.id", std::env::var("KAFKA_CONSUMER_GROUP_NAME")?)
        .set("enable.auto.commit", "true")
        .set("auto.offset.reset", "earliest")
        .set("session.timeout.ms", "45000")
        .create()
        .map_err(|e| ConfigurationError::ConsumerCreateFailed(e))
}

#[derive(Error, Debug)]
pub enum ConsumerError {
    #[error("failed to deserialize message {0}")]
    DeserializationError(#[from] SerDeError),

    #[error("failed to consume message from kafka {0}")]
    KafkaConsumptionFailed(#[from] KafkaError),

    #[error("message payload absent")]
    PayloadAbsent,
}

// A consumer consumes data from a topic. This consumer deserializes each
// message after consuming it, and yields the deserialized message to the
// caller.
pub struct Consumer<T>
where
    T: SerDe,
{
    consumer: StreamConsumer,
    topic: String,
    _t: PhantomData<T>,
}

impl<T: SerDe> Consumer<T> {
    pub fn new(topic: &str) -> Result<Consumer<T>, ConfigurationError> {
        Ok(Consumer {
            consumer: consumer()?,
            topic: topic.to_owned(),
            _t: PhantomData,
        })
    }

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
                    res.map_err(|e| e.into()).and_then(|msg| {
                        T::deserialize(msg.payload().ok_or(ConsumerError::PayloadAbsent)?)
                            .map_err(|e| e.into())
                    })
                })),
            Err(e) => Err(ConfigurationError::SubscriptionFailed(e)),
        }
    }
}

//
// StreamProcessor
//

#[derive(Error, Debug)]
pub enum StreamProcessorError {
    #[error("encountered consumer error {0}")]
    ConsumerError(#[from] ConsumerError),

    #[error("encountered producer error {0}")]
    ProducerError(#[from] ProducerError),

    #[error("encountered event handler error {0}")]
    EventHandlerError(String),
}

// A stream processor consumes data from a topic, does things with the data, and
// produces data to another topic. This stream processor deserializes the data
// after consuming and serializes data before producing.
pub struct StreamProcessor<T, U>
where
    T: SerDe,
    U: SerDe,
{
    consumer: Consumer<T>,
    producer: Producer<U>,
}

impl<T: SerDe, U: SerDe> StreamProcessor<T, U> {
    pub fn new(
        consumer_topic: &str,
        producer_topic: &str,
    ) -> Result<StreamProcessor<T, U>, ConfigurationError> {
        Ok(StreamProcessor {
            consumer: Consumer::new(consumer_topic)?,
            producer: Producer::new(producer_topic)?,
        })
    }

    pub fn stream<'a, F, E>(
        &'a self,
        event_handler: F,
    ) -> Result<impl Stream<Item = Result<(), StreamProcessorError>> + 'a, ConfigurationError>
    where
        F: FnMut(Result<T, StreamProcessorError>) -> Result<U, E> + 'a,
        E: Display,
    {
        Ok(self
            .consumer
            .stream()?
            .map_err(|e| -> StreamProcessorError { e.into() })
            .map(event_handler)
            .map_err(|e| StreamProcessorError::EventHandlerError(e.to_string()))
            .and_then(move |msg| async move {
                // The underlying FutureProducer::clone() call is allegedly
                // inexpensive. It's documented that "It can be cheaply cloned to
                // get a reference to the same underlying producer", and the
                // examples in the rust-rdkafka repo show cloning upon receipt of
                // every message, so I think it's acceptable here?
                self.producer
                    .clone()
                    .send(msg)
                    .map_err(|e| -> StreamProcessorError { e.into() })
                    .await
            }))
    }
}
