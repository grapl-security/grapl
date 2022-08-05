use std::{
    marker::PhantomData,
    sync::{
        Arc,
        Mutex,
    },
    time::Duration,
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
    config::ConsumerConfig,
    Consumer,
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
pub struct KafkaTopicScanner<T>
where
    T: SerDe + Send + Sync + 'static,
{
    consumer_config: ConsumerConfig,
    timeout: Duration,
    t_: PhantomData<T>,
}

impl<T> KafkaTopicScanner<T>
where
    T: SerDe + Send + Sync,
{
    pub fn new(consumer_config: ConsumerConfig, timeout: Duration) -> Self {
        // this overridden config adds a unique suffix to the consumer group,
        // ensuring that each test consumer belongs to its own unique group
        let overridden_config = ConsumerConfig {
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
            consumer_config: overridden_config,
            timeout,
            t_: PhantomData,
        }
    }

    /// Consume messages from Kafka matching the given tenant_id, filtered by
    /// the predicate. This will terminate after max_envelopes have been
    /// consumed. Panics if this takes longer than self.timeout.
    ///
    /// N.B.: If fewer than max_envelopes messages matching the tenant_id and
    /// predicate are available, this will time out.
    pub async fn contains_for_tenant(
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
        self.contains(tenant_eq_predicate, move |idx, _envelope| {
            idx >= max_envelopes
        })
        .await
    }

    /// Consume messages from Kafka matching filter_predicate into a list until
    /// the stop_predicate returns true. Panics if this takes longer than
    /// self.timeout.
    pub async fn contains(
        self,
        filter_predicate: impl FnMut(Envelope<T>) -> bool + Send + Sync + 'static,
        stop_predicate: impl FnMut(usize, Envelope<T>) -> bool + Send + Sync + 'static,
    ) -> JoinHandle<Vec<Envelope<T>>> {
        // we'll use this channel to communicate that the consumer is ready to
        // consume messages
        let (tx, rx) = oneshot::channel::<()>();
        let tx_mutex = Mutex::new(Some(tx));

        tracing::info!("creating kafka subscriber thread");
        let handle = tokio::task::spawn(async move {
            let filter_predicate = Arc::new(Mutex::new(filter_predicate));
            let stop_predicate = Arc::new(Mutex::new(stop_predicate));
            let consumer = Arc::new(
                Consumer::new(self.consumer_config).expect("failed to configure consumer"),
            );

            let filtered_stream = consumer
                .stream()
                .then(move |res| {
                    let (span, envelope) = res.expect("error consuming message from kafka");
                    let _guard = span.enter();

                    tracing::debug!(message = "consumed kafka message");

                    // notify the receiver that the consumer is ready to consume
                    // messages from kafka
                    if let Some(tx) = tx_mutex.lock().expect("failed to acquire tx lock").take() {
                        // notify the channel that we're ready to receive messages
                        if let Err(_) = tx.send(()) {
                            tracing::warn!("receiver was dropped");
                        }
                    }

                    let filter_predicate = filter_predicate.clone();
                    async move {
                        match filter_predicate
                            .lock()
                            .expect("failed to acquire filter predicate lock")(
                            envelope.clone()
                        ) {
                            true => {
                                tracing::debug!("filter predicate matched");
                                Some(envelope)
                            }
                            false => None,
                        }
                    }
                    .instrument(span.clone())
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
                .take_while(|(idx, envelope)| {
                    let stop_predicate = stop_predicate.clone();
                    let idx = *idx;
                    let envelope = envelope.clone();
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
                })
                .map(|(_, envelope)| envelope);

            tracing::info!(
                message = "waiting for kafka messages",
                timeout = ?self.timeout,
            );

            tokio::time::timeout(self.timeout, filtered_stream.collect::<Vec<Envelope<T>>>())
                .await
                .expect("timed out waiting for predicate to match")
        });

        // wait for the kafka consumer to start consuming
        tracing::info!("waiting for kafka consumer to report ready");
        let branch = tokio::select!(
            // If the topic isn't very active, the receiver may never get a
            // notification, so we fall back to a 10 second sleep. A
            // deterministic solution might involve spamming the topic with
            // messages and waiting for the notification, but this seems
            // reliable enough...
            _ = tokio::time::sleep(Duration::from_secs(10)) => {
                "did not receive notification"
            },
            result = rx => {
                result.expect("failed to receive consumer ready notification");
                "received notification"
            }
        );

        tracing::info!(
            message = "kafka consumer is ready to consume messages",
            branch = branch,
        );

        handle
    }
}
