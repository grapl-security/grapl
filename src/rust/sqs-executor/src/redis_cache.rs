use std::{io::Stdout,
          time::{Duration,
                 Instant}};

use async_trait::async_trait;
use darkredis::{ConnectionPool,
                Error as RedisError,
                MSetBuilder};
use futures::FutureExt;
use grapl_observe::{metric_reporter::{tag,
                                      MetricReporter},
                    timers::TimedFutureExt};
use grapl_utils::future_ext::GraplFutureExt;
use lazy_static::lazy_static;
use tokio::time::error::Elapsed;

use crate::{cache::{Cache,
                    CacheResponse,
                    Cacheable},
            errors::{CheckedError,
                     Recoverable}};

lazy_static! {
    /// Timeout for requests to Redis
    static ref REDIS_REQUEST_TIMEOUT: Duration = Duration::from_millis(300);
    /// Expiration value for set Redis keys (after 30 minutes, the key is unset)
    static ref REDIS_SET_EXPIRATION: Duration = Duration::from_secs(30 * 60); // 30 minutes
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
    #[error("Panicked: {0:?}")]
    Panic(Option<String>),
}

impl CheckedError for RedisCacheError {
    fn error_type(&self) -> Recoverable {
        match self {
            RedisCacheError::RedisError(RedisError::EmptySlice) => Recoverable::Persistent,
            _ => Recoverable::Transient,
        }
    }
}

// todo: We should add a circuit breaker to this and not hit the backing redis-store if it closes

#[derive(Clone)]
pub struct RedisCache {
    address: String,
    connection_pool: ConnectionPool,
    metric_reporter: MetricReporter<Stdout>,
    lru_cache: Arc<Mutex<lru::LruCache<Vec<u8>, ()>>>,
}

impl RedisCache {
    pub async fn new(
        address: String,
        metric_reporter: MetricReporter<Stdout>,
    ) -> Result<Self, RedisError> {
        let connection_pool =
            ConnectionPool::create(address.clone(), None, num_cpus::get() * 10).await?;

        Ok(Self {
            connection_pool,
            address,
            metric_reporter,
            lru_cache: Arc::new(Mutex::new(lru::LruCache::new(100_000))),
        })
    }
}

impl RedisCache {
    #[tracing::instrument(skip(self, cacheable))]
    async fn _get<CA>(&mut self, cacheable: CA) -> Result<CacheResponse, RedisCacheError>
    where
        CA: Cacheable + Send + Sync + Clone + 'static,
    {
        self._get_all(vec![cacheable]).await.map(|mut results| {
            results
                .pop()
                .map(|(_, response)| response)
                .unwrap_or(CacheResponse::Miss)
        })
    }

    async fn _get_all<CA>(
        &mut self,
        cacheables: Vec<CA>,
    ) -> Result<Vec<(CA, CacheResponse)>, RedisCacheError>
    where
        CA: Cacheable + Send + Sync + Clone + 'static,
    {
        let cacheable_responses: Vec<(CA, Option<CacheResponse>)> = {
            let mut lru_cache = self.lru_cache.lock().unwrap();

            cacheables
                .into_iter()
                .map(|cacheable| {
                    // if this hits, it's a Some(CacheResponse::Hit)
                    // otherwise, we use None to signify that we should check in with redis
                    let identity_bytes = cacheable.identity();
                    let lru_cache_response =
                        lru_cache.get(&identity_bytes).map(|_| CacheResponse::Hit);

                    (cacheable, lru_cache_response)
                })
                .collect()
        };

        // if the LRU cache satisfied all our needs, we should return early
        let are_all_keys_handled = cacheable_responses
            .iter()
            .all(|(_, response)| response.is_some());

        if are_all_keys_handled {
            return Ok(cacheable_responses
                .into_iter()
                .filter_map(|(cacheable, response)| response.map(|response| (cacheable, response)))
                .collect());
        }

        // fetch the responses from redis and convert them into CacheResponses
        // this Vec<_> should be equal in size to the number of entries in cacheable_responses with
        // 'response' set to None
        let mut unknown_cacheable_responses: VecDeque<CacheResponse> = {
            // this vec will be len() > 0 since we would've returned earlier otherwise
            let unknown_cacheables: Vec<_> = cacheable_responses
                .iter()
                .filter(|(_, response)| response.is_none())
                .map(|(cacheable, _)| cacheable.identity())
                .collect();

            let client_pool = self.connection_pool.clone();
            let mut client = client_pool.get().await;

            client
                .mget(&unknown_cacheables)
                .timeout(REDIS_REQUEST_TIMEOUT.clone())
                .await??
                .into_iter()
                .map(|mget_response: Option<_>| match mget_response {
                    Some(_) => CacheResponse::Hit,
                    None => CacheResponse::Miss,
                })
                .collect()
        };

        // convert original cacheable_responses into a complete set of CacheResponses
        let complete_cacheable_responses: Vec<_> = cacheable_responses
            .into_iter()
            .map(|(cacheable, response)| {
                let actual_response = match response {
                    Some(cache_response) => cache_response,
                    None => unknown_cacheable_responses.pop_front().unwrap_or_else(|| {
                        error!("Missing cacheable response from redis fetch.");
                        CacheResponse::Miss
                    }),
                };

                (cacheable, actual_response)
            })
            .collect();

        if !unknown_cacheable_responses.is_empty() {
            error!("Redis returned more cache responses than expected.");
        }

        Ok(complete_cacheable_responses)
    }

    #[tracing::instrument(skip(self, identity))]
    async fn _store(&mut self, identity: Vec<u8>) -> Result<(), RedisCacheError> {
        // put the key into the cache
        // if it was already in the cache, a Some will return and we know we can return Ok
        if self
            .lru_cache
            .lock()
            .unwrap()
            .put(identity.clone(), LRU_SET_VALUE)
            .is_some()
        {
            return Ok(());
        }

        let identity = hex::encode(identity);

        let mut client = self.connection_pool.get().await;

        client
            .set_and_expire_seconds(
                &identity,
                REDIS_SET_VALUE,
                (*REDIS_SET_EXPIRATION).as_secs() as u32,
            )
            .timeout(REDIS_REQUEST_TIMEOUT.clone())
            .await??;

        Ok(())
    }

    #[tracing::instrument(skip(self, identities))]
    async fn _store_all(&mut self, identities: Vec<Vec<u8>>) -> Result<(), RedisCacheError> {
        if identities.is_empty() {
            return Ok(());
        }

        let identities_to_check: Vec<_> = {
            let mut lru_cache = self.lru_cache.lock().unwrap();

            // for each of the identities:
            //      1. update the lru cache status
            //      2. if key is new (is_none()), collect into a Vec
            identities
                .into_iter()
                .filter(|identity| lru_cache.put(identity.clone(), LRU_SET_VALUE).is_none())
                .collect()
        };

        if identities_to_check.is_empty() {
            return Ok(());
        }

        let client_pool = self.connection_pool.clone();
        let task_start = Instant::now();

        tokio::spawn(async move {
            let mut client = client_pool.get().await;
            let mut mset_builder = MSetBuilder::new();

            for identity in &identities_to_check {
                mset_builder = mset_builder.set(identity, REDIS_SET_VALUE);
            }

            match client.mset(mset_builder).await {
                Ok(_) => Ok(()),
                Err(e) => {
                    let elapsed = task_start.elapsed();

                    if elapsed >= *REDIS_REQUEST_TIMEOUT {
                        let elapsed_ms = elapsed.as_millis() as u64;
                        error!(
                            error = e.to_string().as_str(),
                            elapsed_ms = elapsed_ms,
                            "redis mset failed outside of ttl"
                        );
                        Ok(())
                    } else {
                        Err(e)
                    }
                }
            }
        })
        .timeout(REDIS_REQUEST_TIMEOUT.clone())
        .await???;
        /*
           Three question marks for the following reasons:
           1. Result from timeout
           2. Result from JoinHandle (from tokio::spawn)
           3. Result from `async { ... }` region (we return Ok(()) or Err(..))
        */

        Ok(())
    }
}

use std::{panic::AssertUnwindSafe,
          sync::{Arc,
                 Mutex}};

use prost::alloc::collections::VecDeque;
use tracing::error;

#[async_trait]
impl Cache for RedisCache {
    type CacheErrorT = RedisCacheError;

    #[tracing::instrument(skip(self, cacheable))]
    async fn get<CA>(&mut self, cacheable: CA) -> Result<CacheResponse, Self::CacheErrorT>
    where
        CA: Cacheable + Send + Sync + Clone + 'static,
    {
        let f = async {
            let span = tracing::span!(
                tracing::Level::TRACE,
                "redis_cache.get",
                address=?self.address,
            );
            let _enter = span.enter();
            let (res, ms) = self._get(cacheable).timed().await;

            // todo: Refactor metrics into their own structure
            match res {
                Ok(CacheResponse::Hit) => {
                    tracing::debug!("redis cache hit");
                    self.metric_reporter
                        .histogram(
                            "redis_cache.get.ms",
                            ms as f64,
                            &[tag("success", true), tag("hit", true)],
                        )
                        .unwrap_or_else(|e| error!("failed to report redis_cache.get.ms: {:?}", e));
                    self.metric_reporter
                        .counter(
                            "redis_cache.get.count",
                            1f64,
                            0.10,
                            &[tag("success", true), tag("hit", true)],
                        )
                        .unwrap_or_else(|e| {
                            error!("failed to report redis_cache.get.count: {:?}", e)
                        });
                }
                Ok(CacheResponse::Miss) => {
                    self.metric_reporter
                        .counter(
                            "redis_cache.get.count",
                            1f64,
                            0.10,
                            &[tag("success", true), tag("hit", false)],
                        )
                        .unwrap_or_else(|e| {
                            error!("failed to report redis_cache.get.count: {:?}", e)
                        });
                }
                _ => (),
            }

            res
        };

        let f = AssertUnwindSafe(f);

        f.catch_unwind().await.map_err(|e| {
            if let Some(err) = e.downcast_ref::<&str>() {
                RedisCacheError::Panic(Some(err.to_string()))
            } else if let Some(err) = e.downcast_ref::<String>() {
                RedisCacheError::Panic(Some(err.to_string()))
            } else {
                RedisCacheError::Panic(None)
            }
        })?
    }

    #[tracing::instrument(skip(self, cacheables))]
    async fn get_all<CA>(
        &mut self,
        cacheables: Vec<CA>,
    ) -> Result<Vec<(CA, CacheResponse)>, Self::CacheErrorT>
    where
        CA: Cacheable + Send + Sync + Clone + 'static,
    {
        let f = async {
            let span = tracing::span!(
                tracing::Level::TRACE,
                "redis_cache.get_all",
                address = self.address.as_str(),
                cacheables_len = cacheables.len(),
            );
            let _enter = span.enter();

            let (res, ms) = self._get_all(cacheables).timed().await;

            self.metric_reporter
                .histogram(
                    "redis_cache.get_all.ms",
                    ms as f64,
                    &[tag("success", res.is_ok())],
                )
                .unwrap_or_else(|e| error!("failed to report redis_cache.get_all.ms: {:?}", e));
            res
        };

        let f = AssertUnwindSafe(f);

        f.catch_unwind().await.map_err(|e| {
            if let Some(err) = e.downcast_ref::<&str>() {
                RedisCacheError::Panic(Some(err.to_string()))
            } else if let Some(err) = e.downcast_ref::<String>() {
                RedisCacheError::Panic(Some(err.to_string()))
            } else {
                RedisCacheError::Panic(None)
            }
        })?
    }

    #[tracing::instrument(skip(self, identity))]
    async fn store(&mut self, identity: Vec<u8>) -> Result<(), Self::CacheErrorT> {
        let f = async {
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
        };

        let f = AssertUnwindSafe(f);

        f.catch_unwind().await.map_err(|e| {
            if let Some(err) = e.downcast_ref::<&str>() {
                RedisCacheError::Panic(Some(err.to_string()))
            } else if let Some(err) = e.downcast_ref::<String>() {
                RedisCacheError::Panic(Some(err.to_string()))
            } else {
                RedisCacheError::Panic(None)
            }
        })?
    }

    #[tracing::instrument(skip(self, identities))]
    async fn store_all(&mut self, identities: Vec<Vec<u8>>) -> Result<(), Self::CacheErrorT> {
        let f = async {
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
        };

        let f = AssertUnwindSafe(f);

        f.catch_unwind().await.map_err(|e| {
            if let Some(err) = e.downcast_ref::<&str>() {
                RedisCacheError::Panic(Some(err.to_string()))
            } else if let Some(err) = e.downcast_ref::<String>() {
                RedisCacheError::Panic(Some(err.to_string()))
            } else {
                RedisCacheError::Panic(None)
            }
        })?
    }
}
