use std::time::Duration;

use metric_message::{
    Counter,
    Gauge,
    Histogram,
    MetricWrapper,
};
use metrics::{
    GaugeValue,
    Key,
    Recorder,
    SetRecorderError,
    Unit,
};
use prost::Message;
use rdkafka::{
    client::{
        ClientContext,
        DefaultClientContext,
    },
    producer::{
        FutureProducer,
        FutureRecord as Record,
        Producer,
    },
    util::{
        AsyncRuntime,
        DefaultRuntime,
    },
};
use tokio::sync::mpsc::{
    error::SendError,
    UnboundedReceiver,
    UnboundedSender,
};
use tracing::Instrument;

pub mod counter;
pub mod gauge;
pub mod histogram;
pub mod label;
pub mod metric_message;

pub struct KafkaMetricExporterBuilder<C = DefaultClientContext, R = DefaultRuntime>
where
    C: ClientContext + 'static,
    R: AsyncRuntime,
{
    producer: FutureProducer<C, R>,
    topic_name: String,
}

impl<C, R> KafkaMetricExporterBuilder<C, R>
where
    C: ClientContext + 'static,
    R: AsyncRuntime,
{
    pub fn new(topic_name: impl Into<String>, producer: FutureProducer<C, R>) -> Self {
        Self {
            producer,
            topic_name: topic_name.into(),
        }
    }

    pub fn with_topic_name(&mut self, topic_name: String) -> &mut Self {
        self.topic_name = topic_name;
        self
    }

    pub fn install(self) -> Result<(), SetRecorderError> {
        let (recorder, mut rx) = KafkaRecorder::new();

        tracing::debug!(
            message="Setting up KafkaMetricExporter",
            topic=%self.topic_name,
        );
        metrics::set_boxed_recorder(Box::new(recorder))?;

        let topic = self.topic_name;
        let producer = self.producer;

        tokio::spawn(
            async move {
                let mut payload = Vec::with_capacity(256);
                while let Some(metric) = rx.recv().await {
                    payload.clear();
                    if let Err(e) = metric.payload.encode(&mut payload) {
                        tracing::error!(
                            message="Failed to serialize metric",
                            metric=?metric,
                            error=?e,
                        );
                        continue;
                    };

                    tracing::debug!(message = "Emitting metric as kafka record");
                    let timestamp = Some(metric.timestamp);
                    let key = metric.key.to_be_bytes();

                    let record = Record {
                        topic: &topic,
                        partition: None,
                        payload: Some(payload.as_slice()),
                        key: Some(&key),
                        timestamp,
                        headers: None,
                    };
                    match producer.send(record, Duration::from_secs(3)).await {
                        Ok((partition, offset)) => {
                            tracing::debug!(
                                message="Metric published",
                                topic=%&topic,
                                partition=?partition,
                                offset=?offset,
                            );
                        }
                        Err((e, _)) => {
                            tracing::error!(
                                message="Failed to send message to kafka",
                                topic=%&topic,
                                error=?e,
                            );
                        }
                    }
                }
                producer.flush(Duration::from_secs(3));
            }
            .instrument(tracing::debug_span!("recv loop")),
        );

        Ok(())
    }
}

#[derive(Clone, Debug)]
struct Metric {
    payload: MetricWrapper,
    timestamp: i64,
    key: u64,
}

#[derive(Clone)]
struct MetricsBuffer {
    buffer: UnboundedSender<Metric>,
}

impl MetricsBuffer {
    fn new(buffer: UnboundedSender<Metric>) -> Self {
        Self { buffer }
    }

    fn push(&self, metric: Metric) {
        if let Err(SendError(_)) = self.buffer.send(metric) {
            tracing::warn!(
                message = "Failed to send message to Kafka, MetricsBuffer receiver was dropped",
            );
        };
    }
}

#[derive(Clone)]
struct KafkaRecorder {
    buffered_metrics: MetricsBuffer,
    clock: quanta::Clock,
}

impl KafkaRecorder {
    fn new() -> (Self, UnboundedReceiver<Metric>) {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

        (
            Self {
                buffered_metrics: MetricsBuffer::new(tx),
                clock: quanta::Clock::new(),
            },
            rx,
        )
    }

    fn make_timestamp(&self) -> i64 {
        // Note that this timestamp may be slightly behind reality
        self.clock.recent().as_unix_duration().as_millis() as i64
    }
}

impl Recorder for KafkaRecorder {
    fn register_counter(&self, key: &Key, unit: Option<Unit>, description: Option<&'static str>) {
        tracing::trace!(
            message="register_counter - does nothing for KafkaMetricExporter",
            name=&key.name(),
            unit=?unit,
            description=?description,
        );
    }

    fn register_gauge(&self, key: &Key, unit: Option<Unit>, description: Option<&'static str>) {
        tracing::trace!(
            message="register_gauge - does nothing for KafkaMetricExporter",
            name=&key.name(),
            unit=?unit,
            description=?description,
        );
    }

    fn register_histogram(&self, key: &Key, unit: Option<Unit>, description: Option<&'static str>) {
        tracing::trace!(
            message="register_histogram - does nothing for KafkaMetricExporter",
            name=&key.name(),
            unit=?unit,
            description=?description,
        );
    }

    fn increment_counter(&self, key: &Key, value: u64) {
        tracing::trace!(
            message="increment_counter",
            name=&key.name(),
            labels=?key.labels(),
            value=&value,
        );
        let counter = Counter::from((key, value));
        let metric = Metric {
            payload: counter.into(),
            timestamp: self.make_timestamp(),
            key: key.get_hash(),
        };

        self.buffered_metrics.push(metric);
    }

    fn update_gauge(&self, key: &Key, value: GaugeValue) {
        tracing::trace!(
            message="update_gauge",
            name=&key.name(),
            labels=?key.labels(),
            value=?value,
        );
        let gauge = Gauge::from((key, value));
        let metric = Metric {
            payload: gauge.into(),
            timestamp: self.make_timestamp(),
            key: key.get_hash(),
        };

        self.buffered_metrics.push(metric);
    }

    fn record_histogram(&self, key: &Key, value: f64) {
        tracing::trace!(
            message="update_histogram",
            name=&key.name(),
            labels=?key.labels(),
            value=?value,
        );
        let histogram = Histogram::from((key, value));
        let metric = Metric {
            payload: histogram.into(),
            timestamp: self.make_timestamp(),
            key: key.get_hash(),
        };

        self.buffered_metrics.push(metric);
    }
}
