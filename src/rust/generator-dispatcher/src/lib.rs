use std::time::Duration;

use config::GeneratorDispatcherConfig;
use futures::{
    pin_mut,
    StreamExt,
    TryStreamExt,
};
use generator_ids_cache::{
    GeneratorIdsCache,
    GeneratorIdsCacheError,
};
use kafka::{
    ConfigurationError as KafkaConfigurationError,
    Consumer,
    ConsumerError,
    ProducerError,
    RetryProducer,
};
use rust_proto::{
    graplinc::grapl::{
        api::plugin_work_queue::v1beta1::{
            ExecutionJob,
            PluginWorkQueueServiceClient,
            PluginWorkQueueServiceClientError,
            PushExecuteGeneratorRequest,
        },
        pipeline::{
            v1beta1::RawLog,
            v1beta2::Envelope,
        },
    },
    protocol::service_client::ConnectError,
};
use thiserror::Error;
use uuid::Uuid;

pub mod config;
pub mod generator_ids_cache;

#[derive(Debug, Error)]
pub enum ConfigurationError {
    #[error("error configuring kafka client")]
    KafkaConfigurationError(#[from] KafkaConfigurationError),

    #[error("error configuring generator IDs cache")]
    GeneratorIdsCacheConfigurationError(#[from] ConnectError),
}

#[derive(Debug, Error)]
pub enum GeneratorDispatcherError {
    #[error("plugin registry client error {0}")]
    GeneratorIdsCacheError(#[from] GeneratorIdsCacheError),

    // FIXME: don't crash the service when this happens
    #[error("error sending data to plugin-work-queue {0}")]
    PluginWorkQueueClientError(#[from] PluginWorkQueueServiceClientError),

    // FIXME: also don't crash the service when this happens
    #[error("error trying to send message to kafka {0}")]
    ProducerError(#[from] ProducerError),
}

pub struct GeneratorDispatcher {
    plugin_work_queue_client: PluginWorkQueueServiceClient,
    raw_logs_consumer: Consumer<Envelope<RawLog>>,
    raw_logs_retry_producer: RetryProducer<RawLog>,
    generator_ids_cache: GeneratorIdsCache,
}

impl GeneratorDispatcher {
    #[tracing::instrument(err)]
    pub async fn new(
        config: GeneratorDispatcherConfig,
        plugin_work_queue_client: PluginWorkQueueServiceClient,
    ) -> Result<Self, ConfigurationError> {
        let raw_logs_consumer: Consumer<Envelope<RawLog>> = Consumer::new(config.kafka_config)?;
        let raw_logs_retry_producer: RetryProducer<RawLog> =
            RetryProducer::new(config.kafka_retry_producer_config)?;
        let generator_ids_cache = GeneratorIdsCache::new(
            config.params.generator_ids_cache_capacity,
            Duration::from_millis(config.params.generator_ids_cache_ttl_ms),
            config.params.generator_ids_cache_updater_pool_size,
            config.params.generator_ids_cache_updater_queue_depth,
        )
        .await?;

        Ok(Self {
            plugin_work_queue_client,
            raw_logs_consumer,
            raw_logs_retry_producer,
            generator_ids_cache,
        })
    }

    #[tracing::instrument(skip(self), err)]
    pub async fn run(&mut self, pool_size: usize) -> Result<(), GeneratorDispatcherError> {
        let generator_ids_cache = self.generator_ids_cache.clone();
        let plugin_work_queue_client = self.plugin_work_queue_client.clone();
        let raw_logs_retry_producer = self.raw_logs_retry_producer.clone();

        loop {
            let generator_ids_cache = generator_ids_cache.clone();
            let plugin_work_queue_client = plugin_work_queue_client.clone();
            let raw_logs_retry_producer = raw_logs_retry_producer.clone();

            let buffered = self.raw_logs_consumer
                .stream()
                .take(pool_size)
                .map(move |raw_log_result: Result<Envelope<RawLog>, ConsumerError>| {
                    let generator_ids_cache = generator_ids_cache.clone();
                    let plugin_work_queue_client = plugin_work_queue_client.clone();
                    let raw_logs_retry_producer = raw_logs_retry_producer.clone();

                    async move {
                        match raw_log_result {
                            Ok(envelope) => {
                                match generator_ids_cache
                                    .clone()
                                    .generator_ids_for_event_source(envelope.metadata.event_source_id)
                                    .await
                                {
                                    Ok(Some(generator_ids)) => {
                                        // cache hit
                                        enqueue_plugin_work(
                                            plugin_work_queue_client.clone(),
                                            generator_ids,
                                            envelope
                                        ).await?;

                                        Ok(())
                                    },
                                    Ok(None) => {
                                        // cache miss, but an update was
                                        // successfully enqueued so we'll retry
                                        // the message
                                        retry_message(
                                            &raw_logs_retry_producer,
                                            envelope
                                        ).await?;

                                        Ok(())
                                    },
                                    Err(GeneratorIdsCacheError::Retryable(reason)) => {
                                        // retryable cache error, so we'll retry
                                        // the message
                                        tracing::warn!(
                                            message = "generator IDs cache error, retrying message",
                                            reason =% reason,
                                        );

                                        retry_message(
                                            &raw_logs_retry_producer,
                                            envelope
                                        ).await?;

                                        Ok(())
                                    },
                                    Err(cache_err) => {
                                        // fatal error, bailing out
                                        Err(GeneratorDispatcherError::from(cache_err))
                                    }
                                }
                            },
                            Err(e) => {
                                tracing::error!(
                                    message="error processing kafka message",
                                    reason=%e,
                                );

                                Ok(())
                            }
                        }
                    }
                })
                .buffer_unordered(pool_size);

            pin_mut!(buffered);

            while let Some(result) = buffered.next().await {
                if let Err(e) = result {
                    tracing::error!(
                        message = "fatal error",
                        reason =% e,
                    );
                    return Err(e);
                }
            }
        }
    }
}

async fn retry_message(
    raw_logs_retry_producer: &RetryProducer<RawLog>,
    envelope: Envelope<RawLog>,
) -> Result<(), ProducerError> {
    // TODO: be a little smarter about handling ProducerError here
    raw_logs_retry_producer.send(envelope).await
}

#[tracing::instrument(skip(plugin_work_queue_client, generator_ids, envelope))]
async fn enqueue_plugin_work(
    plugin_work_queue_client: PluginWorkQueueServiceClient,
    generator_ids: Vec<Uuid>,
    envelope: Envelope<RawLog>,
) -> Result<(), GeneratorDispatcherError> {
    let pool_size = generator_ids.len();
    let payload = envelope.inner_message.log_event.clone();
    futures::stream::iter(generator_ids)
        .map(|generator_id| Ok(generator_id))
        .try_for_each_concurrent(pool_size, move |generator_id| {
            let payload = payload.clone();
            let mut plugin_work_queue_client = plugin_work_queue_client.clone();

            async move {
                let execution_job = ExecutionJob {
                    data: payload.clone(),
                };

                tracing::debug!(
                    message = "enqueueing generator execution job",
                    generator_id =% generator_id,
                );

                // TODO: retries, backpressure signalling, etc.
                plugin_work_queue_client
                    .push_execute_generator(PushExecuteGeneratorRequest {
                        execution_job,
                        plugin_id: generator_id,
                    })
                    .await
                    .map(|_| ())
                    .map_err(|e| GeneratorDispatcherError::from(e))
            }
        })
        .await
}
