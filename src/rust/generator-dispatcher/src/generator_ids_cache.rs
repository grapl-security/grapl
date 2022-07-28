use std::{
    sync::Arc,
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
        build_grpc_client_with_options,
        services::PluginRegistryClientConfig,
        BuildGrpcClientOptions,
    },
    graplinc::grapl::api::plugin_registry::v1beta1::{
        GetGeneratorsForEventSourceRequest,
        GetGeneratorsForEventSourceResponse,
        PluginRegistryServiceClientError,
    },
    protocol::{
        service_client::ConnectError,
        status::{
            Code,
            Status,
        },
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
        let plugin_registry_client = Arc::new(tokio::sync::Mutex::new(
            build_grpc_client_with_options(
                client_config,
                BuildGrpcClientOptions {
                    perform_healthcheck: true,
                    ..Default::default()
                },
            )
            .await?,
        ));

        // The updater task is responsible for handling messages on the update
        // queue and querying the plugin-registry for updates corresponding to
        // each message.
        tokio::task::spawn(async move {
            let plugin_registry_client = Arc::clone(&plugin_registry_client);
            let generator_ids_cache = generator_ids_cache.clone();

            updater_rx.for_each_concurrent(
                updater_pool_size,
                move |event_source_id: Uuid| {
                    let plugin_registry_client = Arc::clone(&plugin_registry_client);
                    let generator_ids_cache = generator_ids_cache.clone();

                    async move {
                        let plugin_registry_client = Arc::clone(&plugin_registry_client);
                        let mut client_guard = plugin_registry_client
                            .lock()
                            .await;

                        let generator_ids = match client_guard
                            .get_generators_for_event_source(GetGeneratorsForEventSourceRequest {
                                event_source_id
                            })
                            .await {
                                Ok(response) => {
                                    drop(client_guard); // release the client lock
                                    response.plugin_ids
                                },
                                Err(PluginRegistryServiceClientError::ErrorStatus(Status{
                                    code: Code::NotFound,
                                    ..
                                })) => {
                                    drop(client_guard); // release the client lock
                                    tracing::warn!(
                                        message = "found no generators for event source",
                                        event_source_id =% event_source_id,
                                    );
                                    vec![]
                                },
                                Err(e) => {
                                    // received an error response from the
                                    // plugin-registry service, so we'll retry
                                    // indefinitely using a truncated binary
                                    // exponential backoff with jitter, capped
                                    // at 5s.
                                    drop(client_guard); // release the client lock

                                    let mut result: Result<GetGeneratorsForEventSourceResponse, PluginRegistryServiceClientError> = Err(e);
                                    let mut n = 0;
                                    while let Err(ref e) = result {
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

                                        // acquire client lock
                                        let mut retry_client_guard = plugin_registry_client
                                            .lock()
                                            .await;

                                        result = retry_client_guard
                                            .get_generators_for_event_source(GetGeneratorsForEventSourceRequest {
                                                event_source_id
                                            })
                                            .await;

                                        drop(retry_client_guard); // release the client lock
                                    };

                                    result
                                        .expect("fatal error, unknown state")
                                        .plugin_ids
                                },
                            };

                        if ! generator_ids.is_empty() {
                            generator_ids_cache.insert(
                                event_source_id,
                                generator_ids,
                            ).await;
                        }
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
