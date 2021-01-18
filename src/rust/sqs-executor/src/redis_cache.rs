use std::time::{Duration, Instant};

use darkredis::Error as RedisError;
use darkredis::{ConnectionPool, MSetBuilder};

use tracing::warn;

use async_trait::async_trait;

use crate::cache::{Cache, CacheResponse, Cacheable};
use crate::errors::{CheckedError, Recoverable};
use grapl_observe::metric_reporter::{tag, MetricReporter};
use grapl_observe::timers::TimedFutureExt;
use std::io::Stdout;
use tokio::time::Elapsed;

#[derive(thiserror::Error, Debug)]
pub enum RedisCacheError {
    #[error("RedisError: {0}")]
    RedisError(#[from] RedisError),
    #[error("Cache Timeout: {0}")]
    Timeout(#[from] Elapsed),
    #[error("JoinError: {0}")]
    JoinError(#[from] tokio::task::JoinError),
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
    lru_cache: Arc<Mutex<lru_cache::LruCache<Vec<u8>, ()>>>,
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
            lru_cache: Arc::new(Mutex::new(lru_cache::LruCache::new(100_000))),
        })
    }
}

impl RedisCache {
    #[tracing::instrument(skip(self, cacheable))]
    async fn _get<CA>(&mut self, cacheable: CA) -> Result<CacheResponse, RedisCacheError>
    where
        CA: Cacheable + Send + Sync + 'static,
    {
        let identity_bytes = cacheable.identity();
        {
            let mut lru_cache = self.lru_cache.lock().unwrap();
            if lru_cache.contains_key(&identity_bytes) {
                return Ok(CacheResponse::Hit);
            }
        }

        let identity = hex::encode(&identity_bytes);
        //
        let mut client =
            tokio::time::timeout(Duration::from_secs(1), self.connection_pool.get()).await?;

        let res = tokio::time::timeout(Duration::from_millis(200), client.exists(&identity)).await;

        let res = match res {
            Ok(res) => res,
            Err(e) => {
                warn!(errors = e.to_string().as_str(), "Cache lookup failed with");
                return Ok(CacheResponse::Miss);
            }
        };

        match res {
            Ok(true) => {
                let mut lru_cache = self.lru_cache.lock().unwrap();
                lru_cache.insert(identity_bytes, ());
                Ok(CacheResponse::Hit)
            }
            Ok(false) => Ok(CacheResponse::Miss),
            Err(e) => {
                warn!(error = e.to_string().as_str(), "Cache lookup failed with");
                Ok(CacheResponse::Miss)
            }
        }
    }

    #[tracing::instrument(skip(self, identity))]
    async fn _store(&mut self, identity: Vec<u8>) -> Result<(), RedisCacheError> {
        {
            let mut lru_cache = self.lru_cache.lock().unwrap();
            if lru_cache.contains_key(&identity) {
                return Ok(());
            } else {
                lru_cache.insert(identity.clone(), ());
            }
        }
        let identity = hex::encode(identity);

        let mut client =
            tokio::time::timeout(Duration::from_secs(1), self.connection_pool.get()).await?;

        tokio::time::timeout(
            Duration::from_millis(500),
            client.set_and_expire_seconds(&identity, b"1", 30 * 60),
        )
        .await??;

        Ok(())
    }

    #[tracing::instrument(skip(self, identities))]
    async fn _store_all(&mut self, identities: Vec<Vec<u8>>) -> Result<(), RedisCacheError> {
        if identities.is_empty() {
            return Ok(());
        }
        let mut identities_to_check = Vec::with_capacity(identities.len());
        {
            let mut lru_cache = self.lru_cache.lock().unwrap();
            for identity_bytes in identities.into_iter() {
                if lru_cache.contains_key(&identity_bytes) {
                    continue;
                }
                lru_cache.insert(identity_bytes.clone(), ());
                identities_to_check.push(identity_bytes);
            }
        }
        if identities_to_check.is_empty() {
            return Ok(());
        }
        let identities = identities_to_check;

        let client_pool = self.connection_pool.clone();
        let task_start = Instant::now();

        tokio::time::timeout(
            Duration::from_millis(300),
            tokio::spawn(async move {
                let mut builder = MSetBuilder::new();
                for identity in identities.iter() {
                    builder = builder.set(identity, b"1");
                }
                let mut client = client_pool.get().await;
                let res = client.mset(builder).await;
                match res {
                    Err(e) => {
                        let elapsed = task_start.elapsed();
                        if elapsed >= Duration::from_millis(300) {
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
                    Ok(_) => Ok(()),
                }
            }),
        )
        .await???;

        Ok(())
    }
}

use std::sync::{Arc, Mutex};
use tracing::error;

#[async_trait]
impl Cache for RedisCache {
    type CacheErrorT = RedisCacheError;

    #[tracing::instrument(skip(self, cacheable))]
    async fn get<CA>(&mut self, cacheable: CA) -> Result<CacheResponse, Self::CacheErrorT>
    where
        CA: Cacheable + Send + Sync + 'static,
    {
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
                    .unwrap_or_else(|e| error!("failed to report redis_cache.get.count: {:?}", e));
            }
            Ok(CacheResponse::Miss) => {
                self.metric_reporter
                    .counter(
                        "redis_cache.get.count",
                        1f64,
                        0.10,
                        &[tag("success", true), tag("hit", false)],
                    )
                    .unwrap_or_else(|e| error!("failed to report redis_cache.get.count: {:?}", e));
            }
            _ => (),
        }

        res
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
}
