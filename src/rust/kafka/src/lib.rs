pub mod config;

#[cfg(feature = "test-utils")]
pub mod test_utils;

use std::{
    marker::PhantomData,
    time::{
        Duration,
        SystemTime,
    },
};

use bytes::{
    Bytes,
    BytesMut,
};
use chrono::{
    DateTime,
    Utc,
};
use config::{
    ConsumerConfig,
    ProducerConfig,
    RetryConsumerConfig,
    RetryProducerConfig,
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
    consumer::{
        stream_consumer::StreamConsumer,
        CommitMode,
        Consumer as KafkaConsumer,
    },
    error::KafkaError,
    producer::{
        FutureProducer,
        FutureRecord,
    },
    util::Timeout,
    Message,
};
use rust_proto::{
    graplinc::grapl::pipeline::v1beta1::Envelope,
    SerDe,
    SerDeError,
};
use secrecy::ExposeSecret;
use thiserror::Error;

/// helper function to format a timestamp as ISO-8601 (useful for logging)
pub fn format_iso8601(timestamp: SystemTime) -> String {
    let datetime: DateTime<Utc> = timestamp.into();
    datetime.to_rfc3339()
}

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
    sasl_password: secrecy::SecretString,
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
            .set("sasl.password", sasl_password.expose_secret())
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
    sasl_password: secrecy::SecretString,
) -> Result<FutureProducer, ConfigurationError> {
    configure(bootstrap_servers, sasl_username, sasl_password)
        .set("compression.type", "zstd")
        .set("acks", "all")
        .create()
        .map_err(|e| ConfigurationError::ProducerCreateFailed(e))
}

#[non_exhaustive]
#[derive(Error, Debug, Clone)]
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
    pub async fn send(&self, msg: Envelope<T>) -> Result<(), ProducerError> {
        let tenant_id = msg.tenant_id();
        let trace_id = msg.trace_id();
        let event_source_id = msg.event_source_id();

        let serialized = msg.serialize()?;
        let record: FutureRecord<[u8], [u8]> = FutureRecord::to(&self.topic).payload(&serialized);

        let result = self
            .producer
            .send(record, Timeout::Never)
            .map(|res| -> Result<(), ProducerError> {
                res.map_err(|(e, _)| -> ProducerError { e.into() })
                    .map(|(partition, offset)| {
                        tracing::debug!(
                            message = "wrote kafka message",
                            partition = partition,
                            offset = offset,
                            tenant_id =% tenant_id,
                            trace_id =% trace_id,
                            event_source_id =% event_source_id,
                        );
                    })
            })
            .await;

        result
    }
}

#[derive(Clone)]
pub struct BytesProducer {
    producer: FutureProducer,
    topic: String,
}

impl BytesProducer {
    pub fn new(config: ProducerConfig) -> Result<Self, ConfigurationError> {
        Ok(Self {
            producer: producer(
                config.bootstrap_servers,
                config.sasl_username,
                config.sasl_password,
            )?,
            topic: config.topic,
        })
    }

    #[tracing::instrument(err, skip(self))]
    pub async fn send(&self, msg: Bytes) -> Result<(), ProducerError> {
        let record: FutureRecord<[u8], [u8]> = FutureRecord::to(&self.topic).payload(&msg);

        self.producer
            .send(record, Timeout::Never)
            .map(|res| -> Result<(), ProducerError> {
                res.map_err(|(e, _)| -> ProducerError { e.into() })
                    .map(|(partition, offset)| {
                        tracing::debug!(
                            message = "wrote kafka message",
                            partition = partition,
                            offset = offset,
                        );
                    })
            })
            .await
    }
}

#[derive(Clone)]
pub struct RetryProducer<T>
where
    T: SerDe,
{
    producer: Producer<T>,
}

impl<T: SerDe> RetryProducer<T> {
    pub fn new(config: RetryProducerConfig) -> Result<Self, ConfigurationError> {
        Ok(Self {
            producer: Producer::new(ProducerConfig {
                bootstrap_servers: config.bootstrap_servers,
                sasl_username: config.sasl_username,
                sasl_password: config.sasl_password,
                topic: config.topic,
            })?,
        })
    }

    #[tracing::instrument(err, skip(self))]
    pub async fn send(&self, mut msg: Envelope<T>) -> Result<(), ProducerError> {
        msg.increment_retry_count();
        self.producer.send(msg).await
    }
}

//
// Consumer
//

fn consumer(
    bootstrap_servers: String,
    sasl_username: String,
    sasl_password: secrecy::SecretString,
    consumer_group_name: String,
) -> Result<StreamConsumer, ConfigurationError> {
    configure(bootstrap_servers, sasl_username, sasl_password)
        .set("group.id", consumer_group_name)
        .set("enable.auto.commit", "false")
        .set("enable.auto.offset.store", "true")
        .set("auto.offset.reset", "latest")
        .set("session.timeout.ms", "45000")
        .create()
        .map_err(|e| ConfigurationError::ConsumerCreateFailed(e))
}

#[non_exhaustive]
#[derive(Error, Debug, Clone)]
pub enum ConsumerError {
    #[error("failed to deserialize message {0}")]
    DeserializationError(#[from] SerDeError),

    #[error("failed to consume message from kafka {0}")]
    KafkaConsumptionFailed(#[from] KafkaError),

    #[error("message payload absent")]
    PayloadAbsent,
}

#[non_exhaustive]
#[derive(Error, Debug, Clone)]
pub enum CommitError {
    #[error("failed to commit consumer offsets {0}")]
    CommitFailed(#[from] KafkaError),
}

/// A consumer consumes data from a topic. This consumer deserializes each
/// message after consuming it, and yields the deserialized message to the
/// caller.
pub struct Consumer<T>
where
    T: SerDe,
{
    consumer: StreamConsumer,
    _t: PhantomData<T>,
}

impl<T: SerDe> Consumer<T> {
    pub fn new(config: ConsumerConfig) -> Result<Self, ConfigurationError> {
        let consumer = consumer(
            config.bootstrap_servers,
            config.sasl_username,
            config.sasl_password,
            config.consumer_group_name,
        )?;

        // the .subscribe(..) call must be fully-qualified here because the
        // Consumer name is shadowed in this crate
        if let Err(e) = rdkafka::consumer::Consumer::subscribe(&consumer, &[&config.topic]) {
            return Err(ConfigurationError::SubscriptionFailed(e));
        }

        Ok(Self {
            consumer,
            _t: PhantomData,
        })
    }

    /// Start consuming from Kafka. Returns a stream of deserialized Envelopes
    /// alongside a span which can be used to automatically decorate tracing
    /// data with the Envelope's metadata.
    #[tracing::instrument(skip(self))]
    pub fn stream(
        &self,
    ) -> impl Stream<Item = Result<(tracing::Span, Envelope<T>), ConsumerError>> + '_ {
        self.consumer.stream().then(move |res| async move {
            res.map_err(ConsumerError::from).and_then(move |msg| {
                let deserialized =
                    Envelope::deserialize(msg.payload().ok_or(ConsumerError::PayloadAbsent)?)
                        .map_err(ConsumerError::from);

                deserialized.map(|envelope| {
                    let span = tracing::span!(
                        target: "stream_processor",
                        tracing::Level::INFO,
                        "envelope_span",
                        tenant_id =% envelope.tenant_id(),
                        trace_id =% envelope.trace_id(),
                        event_source_id =% envelope.event_source_id(),
                        retry_count =% envelope.retry_count(),
                        created_time = format_iso8601(envelope.created_time()),
                        last_updated_time = format_iso8601(envelope.last_updated_time()),
                    );

                    (span, envelope)
                })
            })
        })
    }

    #[tracing::instrument(skip(self), err)]
    pub fn commit(&self) -> Result<(), CommitError> {
        Ok(self.consumer.commit_consumer_state(CommitMode::Sync)?)
    }
}

pub struct BytesConsumer {
    consumer: StreamConsumer,
    delay_ms: u64,
}

impl BytesConsumer {
    pub fn new(config: RetryConsumerConfig) -> Result<Self, ConfigurationError> {
        let consumer = consumer(
            config.bootstrap_servers,
            config.sasl_username,
            config.sasl_password,
            config.consumer_group_name,
        )?;

        // the .subscribe(..) call must be fully-qualified here because the
        // Consumer name is shadowed in this crate
        if let Err(e) = rdkafka::consumer::Consumer::subscribe(&consumer, &[&config.topic]) {
            return Err(ConfigurationError::SubscriptionFailed(e));
        }

        Ok(Self {
            consumer,
            delay_ms: config.delay_ms,
        })
    }

    #[tracing::instrument(skip(self))]
    pub fn stream(&self) -> impl Stream<Item = Result<Bytes, ConsumerError>> + '_ {
        self.consumer.stream().then(move |res| async move {
            match res {
                Ok(msg) => {
                    let timestamp = msg.timestamp();
                    if let Some(millis_i64) = timestamp.to_millis() {
                        let millis = if millis_i64 < 0 {
                            0
                        } else {
                            millis_i64.unsigned_abs()
                        };

                        let message_ts = SystemTime::UNIX_EPOCH + Duration::from_millis(millis);
                        let target = message_ts + Duration::from_millis(self.delay_ms);

                        // If the current time is less than the target time we
                        // delay for the duration between the current time and
                        // the target time. In other words, if the target time
                        // has elapsed we continue without delay, otherwise we
                        // wait long enough for it to have elapsed and then
                        // continue.
                        if let Err(e) = target.elapsed() {
                            let duration = e.duration();

                            tracing::debug!(
                                message = "delaying kafka message consumption",
                                kafka_timestamp = format_iso8601(message_ts),
                                delay_ms =% self.delay_ms,
                                delay_duration_ms =% duration.as_millis(),
                            );

                            tokio::time::sleep(duration).await;
                        }
                    } else {
                        tracing::error!("kafka message timestamp unavailable");
                    }

                    let mut buf = BytesMut::with_capacity(msg.payload_len());
                    buf.extend_from_slice(msg.payload().ok_or(ConsumerError::PayloadAbsent)?);
                    Ok(buf.freeze())
                }
                Err(err) => Err(ConsumerError::from(err)),
            }
        })
    }

    #[tracing::instrument(skip(self), err)]
    pub fn commit(&self) -> Result<(), CommitError> {
        Ok(self.consumer.commit_consumer_state(CommitMode::Sync)?)
    }
}

//
// StreamProcessor
//

#[non_exhaustive]
#[derive(Error, Debug, Clone)]
pub enum StreamProcessorError {
    #[error("consumer error {0}")]
    ConsumerError(#[from] ConsumerError),

    #[error("commit error {0}")]
    CommitError(#[from] CommitError),

    #[error("producer error {0}")]
    ProducerError(#[from] ProducerError),

    #[error("event handler error {0}")]
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

    /// Constructs a stream which does the following things:
    ///
    ///   1. Consumes a message from Kafka
    ///   2. Deserializes the message
    ///   3. Applies the event_handler to the message
    ///   4. Publishes the event_handler's results to Kafka
    ///   5. Commits the message offset
    ///
    /// The event_handler is a function which takes inputs with type
    /// Result<(tracing::Span, Envelope<C>), StreamProcessorError> to outputs
    /// with type Stream<Item = Result<Envelope<P>, E>>. The Span is to enable
    /// you to automatically decorate tracing data emitted from your
    /// event_handler with the Envelope's metadata. If your event_handler
    /// returns an empty stream the offset corresponding to the input message
    /// will never be committed, but that's fine so long as eventually a
    /// subsequent call to your event_handler returns a non-empty stream causing
    /// a later offset to be committed.
    ///
    /// N.B.: You must consume this stream serially to ensure proper commit
    /// ordering. Consuming this stream concurrently could result in data loss.
    #[tracing::instrument(skip(self, event_handler))]
    pub fn stream<'a, F, R, E>(
        &'a self,
        event_handler: F,
    ) -> impl Stream<Item = Result<(), StreamProcessorError>> + '_
    where
        F: FnMut(Result<(tracing::Span, Envelope<C>), StreamProcessorError>) -> R + 'a,
        R: Stream<Item = Result<Envelope<P>, E>> + 'a,
        E: Into<StreamProcessorError> + 'a,
    {
        self.consumer
            .stream()
            .map_err(StreamProcessorError::from)
            .map(event_handler)
            .flatten()
            .then(move |result| async move {
                match result {
                    Ok(msg) => {
                        self.producer
                            .clone()
                            .send(msg)
                            .map_err(StreamProcessorError::from)
                            .await
                    }
                    Err(e) => Err(e.into()),
                }
            })
            .then(move |result| async { result.and_then(|_| Ok(self.consumer.commit()?)) })
    }
}

/// A RetryProcessor handles messages on a service's retry topic, imposes a
/// configurable delay, and then sends the message back to the service's main
/// topic. The RetryProcessor does no serialization or deserialization, it just
/// passes the bytes along. This is to prevent a deserialization error, which
/// may have broken the main service, from also breaking the retry service.
pub struct RetryProcessor {
    consumer: BytesConsumer,
    producer: BytesProducer,
}

impl RetryProcessor {
    pub fn new(
        consumer_config: RetryConsumerConfig,
        producer_config: ProducerConfig,
    ) -> Result<RetryProcessor, ConfigurationError> {
        Ok(RetryProcessor {
            consumer: BytesConsumer::new(consumer_config)?,
            producer: BytesProducer::new(producer_config)?,
        })
    }

    /// Constructs a stream which does the following things:
    ///
    ///  1. Consumes a message from Kafka
    ///  2. Inspects the message timestamp, and if necessary pauses for long
    ///  enough to impose this processor's configured delay
    ///  3. Publishes the message to Kafka
    ///  4. Commits the message offset
    ///
    /// N.B.: You must consume this stream serially to ensure proper commit
    /// ordering. Consuming this stream concurrently could result in data loss.
    #[tracing::instrument(skip(self))]
    pub fn stream<'a>(&'a self) -> impl Stream<Item = Result<(), StreamProcessorError>> + '_ {
        self.consumer
            .stream()
            .map_err(StreamProcessorError::from)
            .then(move |result| async move {
                if let Ok(msg) = result {
                    self.producer
                        .clone()
                        .send(msg)
                        .map_err(StreamProcessorError::from)
                        .await
                } else {
                    result.map(|_| ())
                }
            })
            .then(move |result| async { result.and_then(|_| Ok(self.consumer.commit()?)) })
    }
}
