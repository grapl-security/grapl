use std::time::Duration;

use futures::{
    channel::mpsc::{
        Sender,
        TrySendError,
    },
    Future,
    StreamExt,
};
use moka::future::Cache;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AsyncCacheError {
    #[error("fatal error {0}")]
    Fatal(String),

    #[error("retryable error {0}")]
    Retryable(String),
}

impl<K> From<TrySendError<K>> for AsyncCacheError
where
    K: std::hash::Hash + Send + Sync + Eq + std::fmt::Debug,
{
    fn from(try_send_error: TrySendError<K>) -> Self {
        if try_send_error.is_full() {
            AsyncCacheError::Retryable("update queue is full".to_string())
        } else if try_send_error.is_disconnected() {
            AsyncCacheError::Fatal("update queue is disconnected".to_string())
        } else {
            AsyncCacheError::Retryable(format!("unknown TrySendError: {}", try_send_error,))
        }
    }
}

/// A fail-fast, asynchronous, pull-through cache.
///
/// Caches a mapping of {<K>: <V>, ...} in such a way that lookups return
/// immediately, even on cache miss. This may seem counterproductive, but the
/// idea is to use this in a Kafka consumer where we want to avoid waiting for
/// an RPC round-trip while we're holding onto a message. Instead, we dump the
/// message on a retry topic and asynchronously update the cache. That's why
/// this cache returns immediately on both a hit and a miss.
#[derive(Clone, Debug)]
pub struct AsyncCache<K, V>
where
    K: std::hash::Hash + Send + Sync + Eq + std::fmt::Debug + 'static,
    V: Send + Sync + Clone + 'static,
{
    cache: Cache<K, V>,
    updater_tx: Sender<K>,
}

impl<K, V> AsyncCache<K, V>
where
    K: std::hash::Hash + Send + Sync + Eq + std::fmt::Debug + Clone + 'static,
    V: Send + Sync + Clone + 'static,
{
    /// Construct an async cache.
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
    ///
    /// * updater - the function responsible for producing an up-to-date value
    ///   for a given key.
    //#[tracing::instrument]
    pub async fn new<U, F>(
        capacity: u64,
        ttl: Duration,
        updater_pool_size: usize,
        updater_queue_depth: usize,
        updater: U,
    ) -> Self
    where
        U: FnMut(K) -> F + Send + Clone + 'static,
        F: Future<Output = V> + Send,
    {
        let cache = Cache::builder()
            .max_capacity(capacity)
            .time_to_live(ttl)
            .build();

        let cache_retval = cache.clone();

        let (updater_tx, updater_rx) = futures::channel::mpsc::channel(updater_queue_depth);

        // The updater task is responsible for handling messages on the update
        // queue and querying the plugin-registry for updates corresponding to
        // each message.
        tokio::task::spawn(async move {
            updater_rx
                .for_each_concurrent(updater_pool_size, move |key: K| {
                    let cache = cache.clone();
                    let mut updater = updater.clone();

                    async move {
                        let value = updater(key.clone()).await;
                        cache.insert(key, value).await;
                    }
                })
                .await;
        });

        Self {
            cache: cache_retval,
            updater_tx,
        }
    }

    /// Retrieve the value corresponding to the given key.
    ///
    /// This method attempts to resolve the request against a local cache in
    /// memory. Upon cache miss, we attempt to enqueue a cache update and return
    /// immediately.
    ///
    /// # Arguments
    ///
    /// * key - the key to retrieve a value for
    ///
    /// # Returns
    ///
    /// * Ok(Some(value)) - A value if there is an entry present in the cache.
    ///
    /// * Ok(None) - No entry corresponding to the given key was found in the
    ///   cache, but an update request was successfully enqueued.
    ///
    /// * Err(e) - No entry corresponding to the given key was found in the
    ///   cache, and we failed to enqueue an update request.  Failures take two
    ///   forms:
    ///
    ///   - AsyncCacheError::Retryable - e.g. we failed to enqueue an update
    ///     request but it should work eventually if retried.
    ///
    ///   - AsyncCacheError::Fatal - something happened which has "poisoned" the
    ///     AsyncCache instance such that all future calls to this method will
    ///     fail.
    //#[tracing::instrument(skip(self, key))]
    pub async fn get(&mut self, key: K) -> Result<Option<V>, AsyncCacheError> {
        if let Some(value) = self.cache.get(&key) {
            Ok(Some(value)) // cache hit
        } else if let Err(e) = self.updater_tx.try_send(key) {
            Err(e.into()) // cache miss, failed to enqueue an update
        } else {
            Ok(None) // cache miss, successfully enqueued update
        }
    }
}

#[cfg(test)]
mod tests {
    // FIXME
}
