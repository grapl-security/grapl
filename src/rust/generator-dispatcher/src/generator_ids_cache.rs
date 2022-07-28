use std::{
    time::Duration,
};

use clap::Parser;
use futures::{
    channel::mpsc::{
        Sender,
        TrySendError,
    },
    StreamExt,
};
use moka::future::Cache;
use rand::prelude::*;
use rust_proto::{
    client_factory::{
        build_grpc_client,
        services::PluginRegistryClientConfig,
    },
    graplinc::grapl::api::plugin_registry::v1beta1::{
        GetGeneratorsForEventSourceRequest,
        GetGeneratorsForEventSourceResponse,
        PluginRegistryServiceClientError,
    },
    protocol::{
        service_client::ConnectError,
        status::Code,
    },
};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum GeneratorIdsCacheError {
    #[error("fatal error {0}")]
    Fatal(String),

    #[error("retryable error {0}")]
    Retryable(String),
}

impl From<TrySendError<Uuid>> for GeneratorIdsCacheError {
    fn from(try_send_error: TrySendError<Uuid>) -> Self {
        if try_send_error.is_full() {
            GeneratorIdsCacheError::Retryable("update queue is full".to_string())
        } else if try_send_error.is_disconnected() {
            GeneratorIdsCacheError::Fatal("update queue is disconnected".to_string())
        } else {
            GeneratorIdsCacheError::Retryable(format!("unknown TrySendError: {}", try_send_error,))
        }
    }
}

/// A fail-fast, asynchronous, pull-through cache.
///
/// Caches a mapping of {<event_source_id>: [<generator_id>, ...]} in such a way
/// that lookups return immediately, even on cache miss. This may seem
/// counterproductive, but the idea is to use this in a Kafka consumer where we
/// want to avoid waiting for an RPC round-trip while we're holding onto a
/// message. Instead, we dump the message on a retry topic and asynchronously
/// update the cache. That's why this cache returns immediately on both a hit
/// and a miss.
#[derive(Clone, Debug)]
pub struct GeneratorIdsCache {
    generator_ids_cache: Cache<Uuid, Vec<Uuid>>,
    updater_tx: Sender<Uuid>,
}

impl GeneratorIdsCache {
    /// Construct a generator IDs cache.
    ///
    /// # Arguments
    ///
    /// * capacity - the maximum number of keys the cache can hold at any time.
    ///
    /// * ttl - how long to wait before considering a cache entry "stale".
    ///
    /// * updater_pool_size - the maximum number of concurrent tasks for
    ///   handling cache updates.
    ///
    /// * updater_queue_depth - the maximum number of waiting cache updates.
    #[tracing::instrument(err)]
    pub async fn new(
        capacity: u64,
        ttl: Duration,
        updater_pool_size: usize,
        updater_queue_depth: usize,
    ) -> Result<Self, ConnectError> {
        let generator_ids_cache = Cache::builder()
            .max_capacity(capacity)
            .time_to_live(ttl)
            .build();

        let generator_ids_cache_retval = generator_ids_cache.clone();

        let (updater_tx, updater_rx) = futures::channel::mpsc::channel(updater_queue_depth);

        let client_config = PluginRegistryClientConfig::parse();
        let plugin_registry_client = build_grpc_client(client_config).await?;

        // The updater task is responsible for handling messages on the update
        // queue and querying the plugin-registry for updates corresponding to
        // each message.
        tokio::task::spawn(async move {
            let generator_ids_cache = generator_ids_cache.clone();

            updater_rx.for_each_concurrent(
                updater_pool_size,
                |event_source_id: Uuid| {
                    let generator_ids_cache = generator_ids_cache.clone();
                    let mut plugin_registry_client = plugin_registry_client.clone();

                    async move {

                        let generator_ids = match plugin_registry_client
                            .get_generators_for_event_source(GetGeneratorsForEventSourceRequest {
                                event_source_id
                            })
                            .await {
                                Ok(response) => {
                                    response.plugin_ids
                                },
                                Err(e) => {
                                    // received an error response from the
                                    // plugin-registry service, so we'll retry
                                    // indefinitely using a truncated binary
                                    // exponential backoff with jitter, capped
                                    // at 5s.
                                    let mut result: Result<GetGeneratorsForEventSourceResponse, PluginRegistryServiceClientError> = Err(e);
                                    let mut n = 0;
                                    while let Err(ref e) = result {
                                        if let PluginRegistryServiceClientError::ErrorStatus(status) = e {
                                            if let Code::NotFound = status.code() {
                                                tracing::warn!(
                                                    message = "found no generators for event source",
                                                    event_source_id =% event_source_id,
                                                );
                                                break // don't retry NotFound
                                            }
                                        }

                                        n += 1;
                                        let millis = 2_u64.pow(n) + rand::thread_rng()
                                            .gen_range(0..2_u64.pow(n - 1));
                                        let backoff = if millis < 5000 {
                                            Duration::from_millis(millis)
                                        } else {
                                            Duration::from_millis(10000)
                                        };

                                        tracing::error!(
                                            message = "error retrieving generator IDs from plugin-registry",
                                            error =% e,
                                            retry_delay =? backoff,
                                        );

                                        tokio::time::sleep(backoff).await;

                                        result = plugin_registry_client
                                            .get_generators_for_event_source(GetGeneratorsForEventSourceRequest {
                                                event_source_id
                                            })
                                            .await;
                                    };

                                    result
                                        .expect("fatal error, unknown state")
                                        .plugin_ids
                                },
                            };

                        generator_ids_cache.insert(
                            event_source_id,
                            generator_ids,
                        ).await;
                    }
                })
                .await;
        });

        Ok(Self {
            generator_ids_cache: generator_ids_cache_retval,
            updater_tx,
        })
    }

    /// Retrieve the generator IDs corresponding to the given event source ID.
    ///
    /// This method does not reach out to the plugin-registry service's RPC
    /// interface. Instead, it attempts to resolve the request against a local
    /// cache in memory. Upon cache miss, this method attempts to enqueue a
    /// cache update and returns immediately.
    ///
    /// # Arguments
    ///
    /// * event_source_id - the event source to retrieve generator IDs for
    ///
    /// # Returns
    ///
    /// * Ok(Some(generator_ids)) - A vector of generator IDs if there is an
    ///   entry present in the cache.
    ///
    /// * Ok(None) - No entry corresponding to the given event_source_id was
    ///   found in the cache, but an update request was successfully enqueued.
    ///
    /// * Err(e) - No entry corresponding to the given event_source_id was
    ///   found in the cache, and we failed to enqueue an update request.
    ///   Failures take two forms:
    ///
    ///   - GeneratorIdsCacheError::Retryable - e.g. we failed to enqueue an
    ///     update request but it should work eventually if retried.
    ///
    ///   - GeneratorIdsCacheError::Fatal - something happened which has
    ///     "poisoned" the GeneratorIdsCache instance such that all future
    ///     calls to this method will fail.
    #[tracing::instrument(skip(self), err)]
    pub async fn generator_ids_for_event_source(
        &mut self,
        event_source_id: Uuid,
    ) -> Result<Option<Vec<Uuid>>, GeneratorIdsCacheError> {
        if let Some(generator_ids) = self.generator_ids_cache.get(&event_source_id) {
            Ok(Some(generator_ids)) // cache hit
        } else if let Err(e) = self.updater_tx.try_send(event_source_id) {
            Err(e.into()) // cache miss, failed to enqueue an update
        } else {
            Ok(None) // cache miss, successfully enqueued update
        }
    }
}
