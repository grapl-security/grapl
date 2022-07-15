use std::{
    sync::Arc,
    time::{
        Duration,
        SystemTime,
    },
};

use futures::{
    channel::mpsc::{
        Sender,
        TrySendError,
    },
    StreamExt,
};
use lru::LruCache;
use parking_lot::RwLock;
use plugin_registry::client::{
    FromEnv as PRFromEnv,
    PluginRegistryServiceClient,
    PluginRegistryServiceClientError,
};
use rand::prelude::*;
use rust_proto::graplinc::grapl::api::plugin_registry::v1beta1::{
    GetGeneratorsForEventSourceRequest,
    GetGeneratorsForEventSourceResponse,
};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum ConfigurationError {
    #[error("plugin registry client error {0}")]
    PluginRegistryClientError(#[from] PluginRegistryServiceClientError),
}

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

struct GeneratorIdsEntry {
    generator_ids: Vec<Uuid>,
    evict_after: SystemTime,
}

impl GeneratorIdsEntry {
    fn new(generator_ids: Vec<Uuid>, ttl: Duration) -> Self {
        GeneratorIdsEntry {
            generator_ids,
            evict_after: SystemTime::now() + ttl,
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
    generator_ids_cache: Arc<RwLock<LruCache<Uuid, GeneratorIdsEntry>>>,
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
    pub async fn new(
        capacity: usize,
        ttl: Duration,
        updater_pool_size: usize,
        updater_queue_depth: usize,
    ) -> Result<Self, ConfigurationError> {
        let generator_ids_cache: Arc<RwLock<LruCache<Uuid, GeneratorIdsEntry>>> =
            Arc::new(RwLock::new(LruCache::new(capacity)));
        let evictor_generator_ids_cache = generator_ids_cache.clone();
        let updater_generator_ids_cache = generator_ids_cache.clone();

        let (updater_tx, updater_rx) = futures::channel::mpsc::channel(updater_queue_depth);

        // The evictor task is responsible for periodically evicting old entries
        // from the cache.
        tokio::task::spawn(async move {
            let mut delay = tokio::time::interval(ttl);

            loop {
                delay.tick().await;
                // pop entries from the cache in least-recently-used order
                // until either there are no more entries or we have "caught
                // up" to entries which are not yet stale.
                loop {
                    let generator_ids_cache = Arc::clone(&evictor_generator_ids_cache);
                    let cache_read_guard = generator_ids_cache.read();

                    if let Some((event_source_id, entry)) = cache_read_guard.peek_lru() {
                        if SystemTime::now() > entry.evict_after {
                            let mut cache_write_guard = generator_ids_cache.write();
                            cache_write_guard.pop(event_source_id);
                            drop(cache_write_guard);
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
            }
        });

        let plugin_registry_client = Arc::new(tokio::sync::Mutex::new(
            PluginRegistryServiceClient::from_env().await?,
        ));

        // The updater task is responsible for handling messages on the update
        // queue and querying the plugin-registry for updates corresponding to
        // each message.
        tokio::task::spawn(async move {
            let plugin_registry_client = Arc::clone(&plugin_registry_client);

            updater_rx.for_each_concurrent(
                updater_pool_size,
                move |event_source_id: Uuid| {
                    let plugin_registry_client = Arc::clone(&plugin_registry_client);
                    let generator_ids_cache = Arc::clone(&updater_generator_ids_cache);

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
                                Err(e) => {
                                    // received an error response from the
                                    // plugin-registry service, so we'll retry
                                    // indefinitely using a truncated binary
                                    // exponential backoff with jitter, capped
                                    // at 5s.
                                    drop(client_guard); // release the client lock
                                    let mut result: Result<GetGeneratorsForEventSourceResponse, PluginRegistryServiceClientError> = Err(e);
                                    let mut n = 0;
                                    while let Err(e) = result {
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

                        let generator_ids_cache = Arc::clone(&generator_ids_cache);
                        let mut cache_write_guard = generator_ids_cache.write();

                        cache_write_guard.push(
                            event_source_id,
                            GeneratorIdsEntry::new(generator_ids, ttl),
                        );
                    } // release the cache write lock
                })
                .await;
        });

        Ok(Self {
            generator_ids_cache,
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
    ///     update request.
    ///
    ///   - GeneratorIdsCacheError::Fatal - something happened which has
    ///     "poisoned" the GeneratorIdsCache instance such that all future
    ///     calls to this method will fail.
    pub async fn generator_ids_for_event_source(
        &mut self,
        event_source_id: Uuid,
    ) -> Result<Option<Vec<Uuid>>, GeneratorIdsCacheError> {
        let generator_ids_cache = self.generator_ids_cache.clone();
        let cache_read_guard = generator_ids_cache.read();

        if let Some(_) = cache_read_guard.peek(&event_source_id) {
            let mut cache_write_guard = generator_ids_cache.write();

            if let Some(entry) = cache_write_guard.get(&event_source_id) {
                let generator_ids = entry.generator_ids.clone();

                drop(cache_write_guard);

                Ok(Some(generator_ids)) // cache hit!
            } else {
                drop(cache_write_guard);

                if let Err(e) = self.updater_tx.try_send(event_source_id) {
                    Err(e.into()) // cache miss, failed to enqueue an update
                } else {
                    Ok(None) // cache miss, successfully enqueued update
                }
            }
        } else if let Err(e) = self.updater_tx.try_send(event_source_id) {
            Err(e.into()) // cache miss, failed to enqueue an update
        } else {
            Ok(None) // cache miss, successfully enqueued update
        }
    } // release the cache read lock
}
