use std::time::Duration;

use darkredis::ConnectionPool;
use darkredis::Error as RedisError;

use tracing::warn;

use async_trait::async_trait;

use crate::cache::{Cache, CacheResponse, Cacheable};
use crate::errors::{CheckedError, Recoverable};
use grapl_observe::metric_reporter::{tag, MetricReporter};
use grapl_observe::timers::{time_fut_ms, TimedFutureExt};
use std::io::Stdout;
use tokio::time::{Elapsed, Timeout};

#[derive(thiserror::Error, Debug)]
pub enum RedisCacheError {
    #[error("RedisError: {0}")]
    RedisError(#[from] RedisError),
    #[error("Cache Timeout: {0}")]
    Timeout(#[from] Elapsed),
}

impl CheckedError for RedisCacheError {
    fn error_type(&self) -> Recoverable {
        match self {
            RedisCacheError::RedisError(RedisError::EmptySlice) => Recoverable::Persistent,
            _ => Recoverable::Transient,
        }
    }
}

#[derive(Clone)]
pub struct RedisCache {
    address: String,
    connection_pool: ConnectionPool,
    metric_reporter: MetricReporter<Stdout>,
}

impl RedisCache {
    pub async fn new(
        address: String,
        metric_reporter: MetricReporter<Stdout>,
    ) -> Result<Self, RedisError> {
        let connection_pool =
            ConnectionPool::create(address.clone(), None, num_cpus::get()).await?;

        Ok(Self {
            connection_pool,
            address,
            metric_reporter,
        })
    }
}

impl RedisCache {
    #[tracing::instrument(skip(self, cacheable))]
    async fn _get<CA>(&mut self, cacheable: CA) -> Result<CacheResponse, RedisCacheError>
    where
        CA: Cacheable + Send + Sync + 'static,
    {
        let identity = cacheable.identity();
        let identity = hex::encode(identity);
        //
        let mut client = self.connection_pool.get().await;

        let res = tokio::time::timeout(Duration::from_millis(200), client.exists(&identity)).await;

        let res = match res {
            Ok(res) => res,
            Err(e) => {
                warn!("Cache lookup failed with: {:?}", e);
                return Ok(CacheResponse::Miss);
            }
        };

        match res {
            Ok(true) => Ok(CacheResponse::Hit),
            Ok(false) => Ok(CacheResponse::Miss),
            Err(e) => {
                warn!("Cache lookup failed with: {:?}", e);
                Ok(CacheResponse::Miss)
            }
        }
    }

    #[tracing::instrument(skip(self, identity))]
    async fn _store(&mut self, identity: Vec<u8>) -> Result<(), RedisCacheError> {
        let identity = hex::encode(identity);

        let mut client = self.connection_pool.get().await;

        tokio::time::timeout(
            Duration::from_millis(200),
            client.set_and_expire_seconds(&identity, b"1", 16 * 60),
        )
        .await??;

        Ok(())
    }
}

use tracing::error;

#[async_trait]
impl Cache for RedisCache {
    type CacheErrorT = RedisCacheError;
    #[tracing::instrument(skip(self, cacheable))]
    async fn get<CA>(&mut self, cacheable: CA) -> Result<CacheResponse, Self::CacheErrorT>
    where
        CA: Cacheable + Send + Sync + 'static,
    {
        let (res, ms) = self._get(cacheable).timed().await;
        self.metric_reporter
            .histogram(
                "redis_cache.get.ms",
                ms as f64,
                &[tag("success", res.is_ok())],
            )
            .unwrap_or_else(|e| error!("failed to report redis_cache.get.ms: {:?}", e));

        match res {
            Ok(CacheResponse::Hit) => {
                self.metric_reporter
                    .counter(
                        "redis_cache.get.count",
                        1f64,
                        0.10,
                        &[tag("success", res.is_ok()), tag("hit", true)],
                    )
                    .unwrap_or_else(|e| error!("failed to report redis_cache.get.count: {:?}", e));
            }
            Ok(CacheResponse::Miss) => {
                self.metric_reporter
                    .counter(
                        "redis_cache.get.count",
                        1f64,
                        0.10,
                        &[tag("success", res.is_ok()), tag("hit", false)],
                    )
                    .unwrap_or_else(|e| error!("failed to report redis_cache.get.count: {:?}", e));
            }
            _ => (),
        }

        res
    }

    #[tracing::instrument(skip(self, identity))]
    async fn store(&mut self, identity: Vec<u8>) -> Result<(), Self::CacheErrorT> {
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
}
