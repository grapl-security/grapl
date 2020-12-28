use std::time::Duration;

use darkredis::ConnectionPool;
use darkredis::Error as RedisError;

use tracing::warn;

use async_trait::async_trait;

use crate::cache::{Cache, CacheResponse, Cacheable};
use crate::errors::{CheckedError, Recoverable};
use tokio::time::{Timeout, Elapsed};

#[derive(thiserror::Error, Debug)]
pub enum RedisCacheError {
    #[error("RedisError: {0}")]
    RedisError(#[from] RedisError),
    #[error("Cache Timeout: {0}")]
    Timeout(#[from] Elapsed),
}

impl CheckedError for RedisCacheError {
    fn error_type(&self) -> Recoverable {
        Recoverable::Transient
    }
}

pub struct RedisCache {
    address: String,
    connection_pool: ConnectionPool,
}

impl Clone for RedisCache {
    fn clone(&self) -> Self {
        Self {
            address: self.address.clone(),
            connection_pool: self.connection_pool.clone(),
        }
    }
}

impl RedisCache {
    pub async fn new(address: String) -> Result<Self, RedisError> {
        let connection_pool =
            ConnectionPool::create(address.clone(), None, num_cpus::get()).await?;

        Ok(Self {
            connection_pool,
            address,
        })
    }
}

#[async_trait]
impl Cache for RedisCache {
    type CacheErrorT = RedisCacheError;
    #[tracing::instrument(skip(self, cacheable))]
    async fn get<CA>(&mut self, cacheable: CA) -> Result<CacheResponse, Self::CacheErrorT>
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
    async fn store(&mut self, identity: Vec<u8>) -> Result<(), Self::CacheErrorT> {
        let identity = hex::encode(identity);

        let mut client = self.connection_pool.get().await;

        let res = tokio::time::timeout(
            Duration::from_millis(200),
            client.set_and_expire_seconds(&identity, b"1", 16 * 60),
        )
        .await??;

        Ok(())
    }
}
