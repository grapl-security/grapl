use std::{
    marker::PhantomData,
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
/// predicate too permissive! Also, if your timeout is less than 10s and you try
/// to consume from a topic without much activity, you may not end up consuming
/// any messages. See the scan(..) method for more details.
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
    /// the predicate. This returns a JoinHandle to a task which will terminate
    /// after max_envelopes have been consumed or self.timeout has elapsed.
    ///
    /// N.B.: This method may block for up to 10s waiting for activity on the
    /// Kafka topic. Once it consumes the first message the countdown will reset
    /// to make an effort to consume messages continuously for self.timeout
    /// seconds. Make sure you set self.timeout appropriately s.t. it doesn't
    /// elapse before the scanner encounters its first message!
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
    ///
    /// N.B.: This method may block for up to 10s waiting for activity on the
    /// Kafka topic. Once it consumes the first message the countdown will reset
    /// to make an effort to consume messages continuously for self.timeout
    /// seconds. Make sure you set self.timeout appropriately s.t. it doesn't
    /// elapse before the scanner encounters its first message!
    pub async fn scan(
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

            let mut stop_time = SystemTime::now() + self.timeout;

            let filtered_stream = consumer
                .stream()
                .take_until(futures::future::poll_fn(|_ctx| {
                    if let Ok(_) = stop_time.elapsed() {
                        tracing::warn!("timeout elapsed");
                        futures::task::Poll::Ready(())
                    } else {
                        futures::task::Poll::Pending
                    }
                }))
                .then(move |res| {
                    let (span, envelope) = res.expect("error consuming message from kafka");
                    let ret_span = span.clone();
                    let _guard = span.enter();

                    tracing::debug!(message = "consumed kafka message");

                    // notify the receiver that the consumer is ready to consume
                    // messages from kafka
                    if let Some(tx) = tx_mutex.lock().expect("failed to acquire tx lock").take() {
                        // reset the stop_time, because we are now actually
                        // consuming messages, and we want to preserve the
                        // "consume messages for N seconds" semantics of
                        // self.timeout.
                        let mut new_stop_time = SystemTime::now() + self.timeout;
                        std::mem::swap(&mut stop_time, &mut new_stop_time);
                        // notify the channel that we're ready to receive messages
                        if let Err(_) = tx.send(()) {
                            tracing::warn!("receiver was dropped");
                        }
                    }

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

            tracing::info!(
                message = "waiting for kafka messages",
                timeout = ?self.timeout,
            );

            filtered_stream.collect::<Vec<Envelope<T>>>().await
        });

        // wait for the kafka consumer to start consuming
        tracing::info!("waiting for kafka consumer to report ready");
        let branch = tokio::select!(
            // If the topic isn't very active, the receiver may never get a
            // notification, so we fall back to a 10s sleep. A deterministic
            // solution might involve spamming the topic with messages and
            // waiting for the notification, but this seems reliable enough...
            _ = tokio::time::sleep(Duration::from_secs(10)) => {
                "did not receive notification"
            },
            result = rx => {
                result.expect("sender was dropped");
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
