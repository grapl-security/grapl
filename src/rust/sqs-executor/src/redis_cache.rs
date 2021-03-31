use std::{io::Stdout,
          time::Duration};

use async_trait::async_trait;
use grapl_observe::{metric_reporter::{tag,
                                      MetricReporter},
                    timers::TimedFutureExt};
use grapl_utils::future_ext::GraplFutureExt;
use itertools::{Either,
                Itertools};
use lazy_static::lazy_static;
use redis::{AsyncCommands,
            RedisError};
use tokio::time::error::Elapsed;

use crate::{cache::{Cache,
                    Cacheable},
            errors::{CheckedError,
                     Recoverable}};

lazy_static! {
    /// Timeout for requests to Redis
    static ref REDIS_REQUEST_TIMEOUT: Duration = Duration::from_millis(300);
}

/// Value we set in Redis to represent a "set" key
const REDIS_SET_VALUE: &[u8; 1] = b"1";
/// Value we set in LRU cache to represent a "set" key
const LRU_SET_VALUE: () = ();

#[derive(thiserror::Error, Debug)]
pub enum RedisCacheError {
    #[error("RedisError: {0}")]
    RedisError(#[from] RedisError),
    #[error("Cache Timeout")]
    Timeout(#[from] Elapsed),
    #[error("JoinError: {0}")]
    JoinError(#[from] tokio::task::JoinError),
}

impl CheckedError for RedisCacheError {
    fn error_type(&self) -> Recoverable {
        match self {
            _ => Recoverable::Transient,
        }
    }
}

#[derive(Clone)]
pub struct RedisCache {
    address: String,
    connection_manager: redis::aio::ConnectionManager,
    metric_reporter: MetricReporter<Stdout>,
    lru_cache: Arc<Mutex<lru::LruCache<Vec<u8>, ()>>>,
}

impl RedisCache {
    pub async fn new(
        address: String,
        metric_reporter: MetricReporter<Stdout>,
    ) -> Result<Self, RedisError> {
        RedisCache::with_lru_capacity(100_000, address, metric_reporter).await
    }

    pub async fn with_lru_capacity(
        cap: usize,
        address: String,
        metric_reporter: MetricReporter<Stdout>,
    ) -> Result<Self, RedisError> {
        let client = redis::Client::open(address.clone())?;
        let connection_manager = client.get_tokio_connection_manager().await?;

        Ok(Self {
            address,
            connection_manager,
            metric_reporter,
            lru_cache: Arc::new(Mutex::new(lru::LruCache::new(cap))),
        })
    }
}

impl RedisCache {
    async fn _filter_cached<CA>(&mut self, cacheables: Vec<CA>) -> Vec<CA>
    where
        CA: Cacheable + Send + Sync + Clone + 'static,
    {
        let get_identities = |cacheables: &Vec<CA>| -> Vec<Vec<u8>> {
            cacheables.into_iter().map(|c| c.identity()).collect()
        };

        // Check LRU
        let (lru_hits, lru_misses): (Vec<CA>, Vec<CA>) = {
            let mut lru_cache = self.lru_cache.lock().unwrap();

            cacheables
                .into_iter()
                .partition(|c| lru_cache.get(&c.identity()).is_some())
        };

        if !lru_hits.is_empty() {
            info!(
                "Event cache hits from LRU cache: {:?}",
                get_identities(&lru_hits)
            );
        }

        // If we have misses from the LRU cache, we check Redis.
        // If lru_misses is empty we can return with empty vector
        if lru_misses.is_empty() {
            return lru_misses;
        }

        // Check Redis for misses from the LRU cache
        let lru_miss_ids = get_identities(&lru_misses);
        let redis_result: Result<Vec<Option<u8>>, RedisError> =
            self.connection_manager.get(lru_miss_ids).await;

        match redis_result {
            Ok(responses) => {
                let (redis_hits, redis_misses): (Vec<CA>, Vec<CA>) = lru_misses
                    .into_iter()
                    .zip(responses.into_iter())
                    .partition_map(|(c, r)| match r {
                        Some(_) => Either::Left(c),
                        None => Either::Right(c),
                    });

                if !redis_hits.is_empty() {
                    info!(
                        "Event cache hits from Redis: {:?}",
                        get_identities(&redis_hits)
                    );
                }

                // If we have entries in redis_hits here it means they weren't found
                // in the LRU cache. So we'll put them back in LRU cache since they
                // were some of the latest seen
                {
                    let mut lru_cache = self.lru_cache.lock().unwrap();
                    for cacheable in &redis_hits {
                        // We don't care if identity already exists, so we discard the Option returned from put.
                        lru_cache.put(cacheable.identity(), LRU_SET_VALUE);
                    }
                }

                // Return Redis cache missis, which should also be LRU cache misses
                redis_misses
            }
            Err(e) => {
                error!("{:?}", e);
                // If Redis fails, we'll settle for returning LRU cache misses
                lru_misses
            }
        }
    }

    #[tracing::instrument(skip(self, identity))]
    async fn _store(&mut self, identity: Vec<u8>) -> Result<(), RedisCacheError> {
        self._store_all(vec![identity]).await
    }

    #[tracing::instrument(skip(self, identities))]
    async fn _store_all(&mut self, identities: Vec<Vec<u8>>) -> Result<(), RedisCacheError> {
        if identities.is_empty() {
            return Ok(());
        }

        // Attempt to store all identities in LRU and Redis caches

        // LRU PUT
        {
            let mut lru_cache = self.lru_cache.lock().unwrap();
            for identity in &identities {
                // We don't care if identity already exists, so we discard the Option returned from put.
                lru_cache.put(identity.clone(), LRU_SET_VALUE);
            }
        }

        // Redis SET
        let kv_pairs: Vec<(Vec<u8>, &[u8; 1])> = identities
            .clone()
            .into_iter()
            .map(|k| (k, REDIS_SET_VALUE))
            .collect();
        self.connection_manager
            .set_multiple(&kv_pairs)
            .timeout(REDIS_REQUEST_TIMEOUT.clone())
            .await??;

        Ok(())
    }
}

use std::sync::{Arc,
                Mutex};

use tracing::{error,
              info};

#[async_trait]
impl Cache for RedisCache {
    type CacheErrorT = RedisCacheError;

    #[tracing::instrument(skip(self, cacheable))]
    async fn exists<CA: Cacheable + Send + Sync + Clone + 'static>(
        &mut self,
        cacheable: CA,
    ) -> bool {
        let span = tracing::span!(
            tracing::Level::DEBUG,
            "redis_cache.exists",
            address = self.address.as_str(),
        );

        let _enter = span.enter();
        // Here we call filtered_cached to reuse the code for checking LRU and Redis caches
        // If the response is empty then we know the entry is in the cache and we return true
        let (res, ms) = self._filter_cached(vec![cacheable]).timed().await;

        self.metric_reporter
            .histogram("redis_cache.exists.ms", ms as f64, &[tag("success", true)])
            .unwrap_or_else(|e| error!("failed to report redis_cache.exists.ms: {:?}", e));

        // If the response is empty then we know the entry is in the cache and we return true
        res.is_empty()
    }

    #[tracing::instrument(skip(self, identity))]
    async fn store(&mut self, identity: Vec<u8>) -> Result<(), Self::CacheErrorT> {
        let span = tracing::span!(
            tracing::Level::DEBUG,
            "redis_cache.store",
            address = self.address.as_str(),
        );

        let _enter = span.enter();
        let (res, ms) = self._store(identity).timed().await;
        self.metric_reporter
            .histogram(
                "redis_cache.store.ms",
                ms as f64,
                &[tag("success", res.is_ok())],
            )
            .unwrap_or_else(|e| error!("failed to report redis_cache.put.ms: {:?}", e));
        res
    }

    #[tracing::instrument(skip(self, identities))]
    async fn store_all(&mut self, identities: Vec<Vec<u8>>) -> Result<(), Self::CacheErrorT> {
        let span = tracing::span!(
            tracing::Level::TRACE,
            "redis_cache.store_all",
            address = self.address.as_str(),
            identities_len = identities.len(),
        );
        let _enter = span.enter();
        let (res, ms) = self._store_all(identities).timed().await;
        self.metric_reporter
            .histogram(
                "redis_cache.store_all.ms",
                ms as f64,
                &[tag("success", res.is_ok())],
            )
            .unwrap_or_else(|e| error!("failed to report redis_cache.put_all.ms: {:?}", e));
        res
    }

    #[tracing::instrument(skip(self, cacheables))]
    async fn filter_cached<CA>(&mut self, cacheables: Vec<CA>) -> Vec<CA>
    where
        CA: Cacheable + Send + Sync + Clone + 'static,
    {
        let span = tracing::span!(
            tracing::Level::TRACE,
            "redis_cache.filter_cached",
            address = self.address.as_str(),
            cacheables_len = cacheables.len(),
        );
        let _enter = span.enter();

        let (res, ms) = self._filter_cached(cacheables).timed().await;

        self.metric_reporter
            .histogram(
                "redis_cache.filter_cached.ms",
                ms as f64,
                &[tag("success", true)],
            )
            .unwrap_or_else(|e| error!("failed to report redis_cache.filter_cached.ms: {:?}", e));

        res
    }
}
