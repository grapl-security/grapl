use std::time::Duration;

use async_cache::{
    AsyncCache,
    AsyncCacheError,
};
use clap::Parser;
use config::GeneratorDispatcherConfig;
use futures::{
    pin_mut,
    StreamExt,
    TryStreamExt,
};
use kafka::{
    CommitError,
    ConfigurationError as KafkaConfigurationError,
    Consumer,
    ConsumerError,
    ProducerError,
    RetryProducer,
};
use rust_proto::graplinc::grapl::{
    api::{
        client_factory::{
            build_grpc_client,
            services::PluginRegistryClientConfig,
        },
        plugin_registry::v1beta1::GetGeneratorsForEventSourceRequest,
        plugin_work_queue::v1beta1::{
            ExecutionJob,
            PluginWorkQueueServiceClient,
            PluginWorkQueueServiceClientError,
            PushExecuteGeneratorRequest,
        },
        protocol::{
            error::GrpcClientError,
            service_client::ConnectError,
            status::{
                Code,
                Status,
            },
        },
    },
    pipeline::v1beta1::{
        Envelope,
        RawLog,
    },
};
use thiserror::Error;
use tracing::Instrument;
use uuid::Uuid;

pub mod config;

#[derive(Debug, Error)]
pub enum ConfigurationError {
    #[error("error configuring kafka client")]
    KafkaConfigurationError(#[from] KafkaConfigurationError),

    #[error("error configuring generator IDs cache")]
    GeneratorIdsCacheConfigurationError(#[from] ConnectError),
}

#[derive(Debug, Error)]
pub enum GeneratorDispatcherError {
    #[error("generator IDs cache error {0}")]
    GeneratorIdsCacheError(#[from] AsyncCacheError),

    // FIXME: don't crash the service when this happens
    #[error("error sending data to plugin-work-queue {0}")]
    PluginWorkQueueClientError(#[from] PluginWorkQueueServiceClientError),

    // FIXME: also don't crash the service when this happens
    #[error("error trying to send message to kafka {0}")]
    ProducerError(#[from] ProducerError),

    #[error("error committing kafka consumer offsets {0}")]
    CommitError(#[from] CommitError),
}

pub struct GeneratorDispatcher {
    plugin_work_queue_client: PluginWorkQueueServiceClient,
    raw_logs_consumer: Consumer<RawLog>,
    raw_logs_retry_producer: RetryProducer<RawLog>,
    generator_ids_cache: AsyncCache<Uuid, Vec<Uuid>>,
}

impl GeneratorDispatcher {
    #[tracing::instrument(skip(plugin_work_queue_client), err)]
    pub async fn new(
        config: GeneratorDispatcherConfig,
        plugin_work_queue_client: PluginWorkQueueServiceClient,
    ) -> Result<Self, ConfigurationError> {
        let raw_logs_consumer: Consumer<RawLog> = Consumer::new(config.kafka_config)?;
        let raw_logs_retry_producer: RetryProducer<RawLog> =
            RetryProducer::new(config.kafka_retry_producer_config)?;
        let client_config = PluginRegistryClientConfig::parse();
        let plugin_registry_client = build_grpc_client(client_config).await?;
        let generator_ids_cache = AsyncCache::new(
            config.params.generator_ids_cache_capacity,
            Duration::from_millis(config.params.generator_ids_cache_ttl_ms),
            config.params.generator_ids_cache_updater_pool_size,
            config.params.generator_ids_cache_updater_queue_depth,
            move |event_source_id| {
                let mut plugin_registry_client = plugin_registry_client.clone();

                async move {
                    match plugin_registry_client
                        .get_generators_for_event_source(GetGeneratorsForEventSourceRequest::new(
                            event_source_id,
                        ))
                        .await
                    {
                        Ok(response) => Some(response.plugin_ids().to_vec()),
                        Err(GrpcClientError::ErrorStatus(Status {
                            code: Code::NotFound,
                            ..
                        })) => {
                            tracing::warn!(
                                message = "found no generators for event source",
                                event_source_id =% event_source_id,
                            );
                            Some(vec![])
                        }
                        Err(e) => {
                            // failed to update the cache, but the message will
                            // be retried via the kafka retry topic
                            tracing::error!(
                                message = "error retrieving generators for event source",
                                event_source_id =% event_source_id,
                                reason =% e,
                            );

                            None
                        }
                    }
                }
            },
        )
        .await;

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

            let stream = self.raw_logs_consumer
                .stream()
                .take(pool_size)
                .then(move |raw_log_result: Result<(tracing::Span, Envelope<RawLog>), ConsumerError>| {
                    let generator_ids_cache = generator_ids_cache.clone();
                    let plugin_work_queue_client = plugin_work_queue_client.clone();
                    let raw_logs_retry_producer = raw_logs_retry_producer.clone();

                    async move {
                        match raw_log_result {
                            Ok((span, envelope)) => {
                                match generator_ids_cache
                                    .clone()
                                    .get(envelope.event_source_id())
                                    .instrument(span.clone())
                                    .await
                                {
                                    Ok(Some(generator_ids)) => {
                                        if generator_ids.is_empty() {
                                            let _guard = span.enter();
                                            tracing::warn!(
                                                message = "no generators for event source",
                                            );
                                        } else {
                                            // cache hit
                                            enqueue_plugin_work(
                                                plugin_work_queue_client.clone(),
                                                generator_ids,
                                                envelope
                                            )
                                                .instrument(span)
                                                .await?;
                                        }

                                        Ok(())
                                    },
                                    Ok(None) => {
                                        // cache miss, but an update was
                                        // successfully enqueued so we'll retry
                                        // the message
                                        let _guard = span.enter();
                                        tracing::debug!(
                                            message = "generator IDs cache miss, retrying message",
                                        );
                                        drop(_guard);

                                        retry_message(
                                            &raw_logs_retry_producer,
                                            envelope
                                        ).instrument(span).await?;

                                        Ok(())
                                    },
                                    Err(AsyncCacheError::Retryable(reason)) => {
                                        // retryable cache error, so we'll retry
                                        // the message
                                        let _guard = span.enter();
                                        tracing::warn!(
                                            message = "generator IDs cache error, retrying message",
                                            reason =% reason,
                                        );
                                        drop(_guard);

                                        retry_message(
                                            &raw_logs_retry_producer,
                                            envelope
                                        )
                                            .instrument(span)
                                            .await?;

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
                .then(|result| async {
                    self.raw_logs_consumer.commit()?;
                    result
                });

            pin_mut!(stream);

            while let Some(result) = stream.next().await {
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

#[tracing::instrument(skip(plugin_work_queue_client, generator_ids, envelope), err)]
async fn enqueue_plugin_work(
    plugin_work_queue_client: PluginWorkQueueServiceClient,
    generator_ids: Vec<Uuid>,
    envelope: Envelope<RawLog>,
) -> Result<(), GeneratorDispatcherError> {
    let pool_size = generator_ids.len();
    let tenant_id = envelope.tenant_id();
    let trace_id = envelope.trace_id();
    let event_source_id = envelope.event_source_id();
    let payload = envelope.inner_message().log_event();
    futures::stream::iter(generator_ids)
        .map(|generator_id| Ok(generator_id))
        .try_for_each_concurrent(pool_size, move |generator_id| {
            let payload = payload.clone();
            let mut plugin_work_queue_client = plugin_work_queue_client.clone();

            async move {
                let execution_job =
                    ExecutionJob::new(payload.clone(), tenant_id, trace_id, event_source_id);

                tracing::debug!(
                    message = "enqueueing generator execution job",
                    generator_id =% generator_id,
                );

                // TODO: retries, backpressure signalling, etc.
                plugin_work_queue_client
                    .push_execute_generator(PushExecuteGeneratorRequest::new(
                        execution_job,
                        generator_id,
                    ))
                    .await
                    .map(|_| ())
                    .map_err(|e| GeneratorDispatcherError::from(e))
            }
        })
        .await
}
