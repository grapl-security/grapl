use std::{
    sync::{
        Arc,
        Mutex,
    },
    time::{
        Duration,
        SystemTime,
    },
};

use futures::StreamExt;
use rust_proto::{
    graplinc::grapl::pipeline::v1beta1::Envelope,
    SerDe,
};
use tokio::{
    sync::oneshot,
    task::JoinHandle,
};
use tracing::Instrument;

use crate::{
    config::{
        ConsumerConfig,
        ProducerConfig,
    },
    Consumer,
    Producer,
};

/// Usage:
///
///   1. Construct a KafkaTopicScanner with a predicate to filter events.
///   2. Call `.contains*().await` to receive a JoinHandle. Once you receive the
///   JoinHandle you can be confident the scanner is consuming messages from the
///   topic.
///   3. Do something that'll result in logs showing up in Kafka.
///   4. `.await?` the JoinHandle to receive all the matching events (or a
///   JoinError containing information about any panics that may have occurred).
///   5. Make your test assertions on this list of matching events.
///
/// N.B.: These results will be materialized in memory, so don't make your
/// predicate too permissive!
///
/// Arguments:
///
/// - consumer_config: The configuration for this scanner's Kafka consumer
///
/// - timeout: Duration for which we'll consume messages before terminating the
///   stream.
///
/// - priming_message: A throw-away message that will be used to "prime" this
///   Kafka consumer. You should make this message invisible to your tests by
///   ensuring it has a unique tenant_id that is different from your test data's
///   tenant_id. This message will be sent to the topic every 500ms and the
///   KafkaTopicScanner will only report "ready" upon first receipt of this
///   message. Be judicious in how you construct this message, as other
///   listeners on the topic will receive it also!
pub struct KafkaTopicScanner<T>
where
    T: SerDe + Send + Sync + 'static,
{
    producer_config: ProducerConfig,
    consumer_config: ConsumerConfig,
    timeout: Duration,
    priming_message: Envelope<T>,
}

impl<T> KafkaTopicScanner<T>
where
    T: SerDe + Send + Sync,
{
    pub fn new(
        consumer_config: ConsumerConfig,
        timeout: Duration,
        priming_message: Envelope<T>,
    ) -> Self {
        // this overridden config adds a unique suffix to the consumer group,
        // ensuring that each test consumer belongs to its own unique group
        let overridden_consumer_config = ConsumerConfig {
            bootstrap_servers: consumer_config.bootstrap_servers,
            sasl_username: consumer_config.sasl_username,
            sasl_password: consumer_config.sasl_password,
            consumer_group_name: format!(
                "{}-{}",
                consumer_config.consumer_group_name,
                uuid::Uuid::new_v4(),
            ),
            topic: consumer_config.topic,
        };

        Self {
            producer_config: overridden_consumer_config.clone().into(),
            consumer_config: overridden_consumer_config,
            timeout,
            priming_message,
        }
    }

    /// Consume messages from Kafka matching the given tenant_id, filtered by
    /// the predicate. This returns a JoinHandle to a task which will terminate
    /// after max_envelopes have been consumed or self.timeout has elapsed.
    #[tracing::instrument(skip(self, predicate))]
    pub async fn scan_for_tenant(
        self,
        tenant_id: uuid::Uuid,
        max_envelopes: usize,
        mut predicate: impl FnMut(T) -> bool + Send + Sync + 'static,
    ) -> JoinHandle<Vec<Envelope<T>>> {
        let tenant_eq_predicate = move |envelope: Envelope<T>| {
            let envelope_tenant_id = envelope.tenant_id();
            let inner_message = envelope.inner_message();
            envelope_tenant_id == tenant_id && predicate(inner_message)
        };
        self.scan(tenant_eq_predicate, move |idx, _envelope| {
            idx >= max_envelopes
        })
        .await
    }

    /// Consume messages from Kafka matching filter_predicate into a list until
    /// the stop_predicate returns true or self.timeout has elapsed. Returns a
    /// JoinHandle to the task which will terminate when these conditions are
    /// met.
    #[tracing::instrument(skip(self, filter_predicate, stop_predicate))]
    pub async fn scan(
        self,
        filter_predicate: impl FnMut(Envelope<T>) -> bool + Send + Sync + 'static,
        stop_predicate: impl FnMut(usize, Envelope<T>) -> bool + Send + Sync + 'static,
    ) -> JoinHandle<Vec<Envelope<T>>> {
        // we'll use this channel to communicate that the consumer is ready to
        // consume messages
        let (tx, rx) = oneshot::channel::<()>();
        let tx_mutex = Mutex::new(Some(tx));
        let priming_message_tenant_id = self.priming_message.tenant_id();

        tracing::info!("creating kafka subscriber thread");
        let handle = tokio::task::spawn(async move {
            let filter_predicate = Arc::new(Mutex::new(filter_predicate));
            let stop_predicate = Arc::new(Mutex::new(stop_predicate));
            let consumer = Arc::new(
                Consumer::new(self.consumer_config).expect("failed to configure consumer"),
            );

            let stop_time: Mutex<Option<SystemTime>> = Mutex::new(None);
            let filtered_stream = consumer
                .stream()
                .then(|res| {
                    let (span, envelope) = res.expect("error consuming message from kafka");
                    let ret_span = span.clone();
                    let _guard = span.enter();

                    tracing::debug!(message = "consumed kafka message");

                    if envelope.tenant_id() == priming_message_tenant_id {
                        tracing::info!("received priming message");

                        // notify the receiver that the consumer is ready to
                        // consume messages from kafka
                        if let Some(tx) = tx_mutex.lock().expect("failed to acquire tx lock").take()
                        {
                            // reset the stop_time, because we are now actually
                            // consuming messages, and we want to preserve the
                            // "consume messages for N seconds" semantics of
                            // self.timeout.
                            let mut stop_time_guard =
                                stop_time.lock().expect("failed to acquire stop_time lock");
                            let new_stop_time =
                                stop_time_guard.insert(SystemTime::now() + self.timeout);

                            // notify the channel that we're ready to receive
                            // messages
                            tracing::info!(
                                message = "starting countdown",
                                timeout =? self.timeout,
                                stop_time =? new_stop_time,
                            );
                            tx.send(()).expect("receiver was dropped");
                        }
                    }

                    async { (ret_span, envelope) }
                })
                .take_until(futures::future::poll_fn(|_ctx| {
                    let stop_time_guard =
                        stop_time.lock().expect("failed to acquire stop_time lock");
                    if let Some(stop_time) = stop_time_guard.as_ref() {
                        if let Ok(_) = stop_time.elapsed() {
                            tracing::warn!("timeout elapsed");
                            futures::task::Poll::Ready(())
                        } else {
                            futures::task::Poll::Pending
                        }
                    } else {
                        futures::task::Poll::Pending
                    }
                }))
                .then(move |(span, envelope)| {
                    let ret_span = span.clone();
                    let filter_predicate = filter_predicate.clone();

                    async move {
                        if filter_predicate
                            .lock()
                            .expect("failed to acquire filter predicate lock")(
                            envelope.clone()
                        ) {
                            tracing::debug!("filter predicate matched");
                            Some((ret_span, envelope))
                        } else {
                            None
                        }
                    }
                    .instrument(span)
                })
                .then(|matched| {
                    let consumer = consumer.clone();
                    async move {
                        consumer.commit().expect("failed to commit consumer offset");
                        matched
                    }
                })
                .filter_map(|matched| async move { matched })
                .enumerate()
                .take_while(|(idx, (span, envelope))| {
                    let stop_predicate = stop_predicate.clone();
                    let idx = *idx;
                    let envelope = envelope.clone();
                    let span = span.clone();
                    async move {
                        if stop_predicate
                            .lock()
                            .expect("failed to acquire stop predicate lock")(
                            idx, envelope
                        ) {
                            tracing::debug!("stop predicate matched");
                            false
                        } else {
                            true
                        }
                    }
                    .instrument(span)
                })
                .map(|(_, (_, envelope))| envelope);

            filtered_stream.collect::<Vec<Envelope<T>>>().await
        });

        // send the self.priming_message every 0.5s until we catch it
        let priming_message = self.priming_message.clone();
        let producer_config = self.producer_config.clone();
        let (primer_tx, primer_rx) = oneshot::channel::<()>();

        let primer_handle = tokio::task::spawn(async move {
            let priming_message = priming_message.clone();
            let producer: Producer<T> =
                Producer::new(producer_config.clone()).expect("failed to configure producer");
            let mut interval = tokio::time::interval(Duration::from_millis(500));

            // wait for whichever completes first--we receive a termination
            // signal on primer_rx or the sender loop errors out
            tokio::select!(
                primer_rx_result = primer_rx => {
                    primer_rx_result.expect("primer_tx sender was dropped");
                },
                _ = async move {
                    loop {
                        interval.tick().await;
                        let producer = producer.clone();

                        tracing::info!(
                            message = "sending priming message",
                            tenant_id =% priming_message.tenant_id(),
                        );

                        if let Err(e) = producer
                            .send(priming_message.clone())
                            .await
                        {
                            tracing::warn!(
                                message = "error sending priming message",
                                tenant_id =% priming_message.tenant_id(),
                                reason =% e,
                            )
                        } else {
                            tracing::info!(
                                message = "sent priming message",
                                tenant_id =% priming_message.tenant_id(),
                            );
                        }
                    }
                } => {
                    // nothing to see here
                }
            );
        });

        // wait for the kafka consumer to start consuming
        tracing::info!("waiting for kafka consumer to report ready");

        tokio::select!(
            rx_result = rx => {
                primer_tx.send(()).expect("failed to notify primer_tx to shutdown");
                rx_result.expect("sender was dropped");
            },
            primer_result = primer_handle => {
                primer_result.expect("primer failed");
            }
        );

        tracing::info!(message = "kafka consumer is ready to consume messages",);

        handle
    }
}
