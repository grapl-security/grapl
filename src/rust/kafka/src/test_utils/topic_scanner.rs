use std::{
    sync::Mutex,
    time::Duration,
};

use futures::StreamExt;
use rust_proto::{
    graplinc::grapl::pipeline::v1beta1::Envelope,
    SerDe,
};
use tokio::{
    sync::{
        oneshot,
        oneshot::error::RecvError,
    },
    task::JoinError,
    time::error::Elapsed,
};

use crate::{
    config::ConsumerConfig,
    ConfigurationError,
    Consumer,
    ConsumerError,
};

#[derive(thiserror::Error, Debug)]
pub enum KafkaTopicContainsError {
    #[error("Timed out looking for a message matching this predicate: {0}")]
    KafkaTopicContainsElapsed(#[from] Elapsed),
    #[error("RecvError: failed to receive notification that consumer is consuming")]
    RecvError(#[from] RecvError),
    #[error("JoinError: could not join kafka subscriber")]
    JoinError(#[from] JoinError),
}

/// Usage:
/// 1. Construct a KafkaTopicScanner
/// 2. Call `.contains*().await`
/// 3. Kick off an event that'll result in logs showing up in Kafka
/// 4. Call `.get_listen_result().await` to wait for the first matching element
pub struct KafkaTopicScanner<T>
where
    T: SerDe + Send + Sync + 'static,
{
    consumer: Consumer<Envelope<T>>,
    timeout: Duration,
}

impl<T> KafkaTopicScanner<T>
where
    T: SerDe + Send + Sync,
{
    pub fn new(consumer_config: ConsumerConfig) -> Result<Self, ConfigurationError> {
        let consumer = Consumer::new(consumer_config)?;
        let timeout = Duration::from_secs(30); // hardcoded for now, whatever
        Ok(Self { consumer, timeout })
    }

    pub async fn contains_for_tenant(
        self,
        tenant_id: uuid::Uuid,
        mut predicate: impl FnMut(T) -> bool + Send + Sync + 'static,
    ) -> Result<
        ScanReadyToGetResult<Result<Envelope<T>, KafkaTopicContainsError>>,
        KafkaTopicContainsError,
    > {
        let tenant_eq_predicate = move |envelope: Envelope<T>| {
            let envelope_tenant_id = envelope.tenant_id();
            let inner_message = envelope.inner_message();
            envelope_tenant_id == tenant_id && predicate(inner_message)
        };
        self.contains(tenant_eq_predicate).await
    }

    pub async fn contains(
        self,
        predicate: impl FnMut(Envelope<T>) -> bool + Send + Sync + 'static,
    ) -> Result<
        ScanReadyToGetResult<Result<Envelope<T>, KafkaTopicContainsError>>,
        KafkaTopicContainsError,
    > {
        // we'll use this channel to communicate that the consumer is ready to
        // consume messages
        let (tx, rx) = oneshot::channel::<()>();

        let predicate = std::sync::Arc::new(std::sync::Mutex::new(predicate));

        tracing::info!("creating kafka subscriber thread");
        let kafka_subscriber = tokio::task::spawn(async move {
            let stream = Box::pin(self.consumer.stream());

            let tx_mutex = Mutex::new(Some(tx));
            let mut filtered_stream = Box::pin(stream.filter_map(
                move |res: Result<Envelope<T>, ConsumerError>| {
                    if let Some(tx) = tx_mutex.lock().expect("failed to acquire tx lock").take() {
                        // notify the consumer that we're ready to receive messages
                        tx.send(())
                            .expect("failed to notify sender that consumer is consuming");
                    }

                    let predicate = predicate.clone();
                    async move {
                        let envelope = res.expect("error consuming message from kafka");
                        match predicate.clone().lock().expect("locking")(envelope.clone()) {
                            true => Some(envelope),
                            false => None,
                        }
                    }
                },
            ));
            let matched_predicate = filtered_stream.next();

            tracing::info!(
                message = "Consuming kafka messages",
                timeout = ?self.timeout,
            );
            let matched_predicate = tokio::time::timeout(self.timeout, matched_predicate).await?;
            Ok(matched_predicate
                .expect("Can't occur - if this were None we'd have the above timeout"))
        });

        // wait for the kafka consumer to start consuming
        tracing::info!("waiting for kafka consumer to report ready");
        rx.await?;
        Ok(ScanReadyToGetResult {
            listen_result: kafka_subscriber,
        })
    }
}

pub struct ScanReadyToGetResult<T> {
    listen_result: tokio::task::JoinHandle<T>,
}
impl<T> ScanReadyToGetResult<T> {
    pub async fn get_listen_result(self) -> Result<T, KafkaTopicContainsError> {
        Ok(self.listen_result.await?)
    }
}
