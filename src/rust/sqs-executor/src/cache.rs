use std::{
    collections::hash_map::DefaultHasher,
    hash::{
        Hash,
        Hasher,
    },
};

use async_trait::async_trait;

use crate::errors::{
    CheckedError,
    Recoverable,
};

pub trait Cacheable {
    fn identity(&self) -> Vec<u8>;
}

impl<H> Cacheable for H
where
    H: Hash,
{
    fn identity(&self) -> Vec<u8> {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        let hash = hasher.finish();
        hash.to_le_bytes().to_vec()
    }
}

#[async_trait]
pub trait Cache: Clone {
    type CacheErrorT: CheckedError + Send + Sync + 'static;

    async fn all_exist<CA>(&mut self, cacheables: &[CA]) -> bool
    where
        CA: Cacheable + Send + Sync + Clone + 'static,
    {
        // If cacheable doesn't return from filter_cached then
        // we know it exists in the cache
        self.filter_cached(cacheables).await.is_empty()
    }

    async fn store<CA>(&mut self, cacheable: CA) -> Result<(), Self::CacheErrorT>
    where
        CA: Cacheable + Send + Sync + Clone + 'static;

    async fn store_all<CA>(&mut self, cacheables: &[CA]) -> Result<(), Self::CacheErrorT>
    where
        CA: Cacheable + Send + Sync + Clone + 'static,
    {
        for cacheable in cacheables.into_iter() {
            self.store(cacheable.identity()).await?;
        }
        Ok(())
    }

    async fn filter_cached<CA>(&mut self, cacheables: &[CA]) -> Vec<CA>
    where
        CA: Cacheable + Send + Sync + Clone + 'static;
}

#[derive(thiserror::Error, Debug)]
pub enum NopCacheError {
    #[error("NopCache never errors")]
    Never,
}

impl CheckedError for NopCacheError {
    fn error_type(&self) -> Recoverable {
        panic!("NopCache can not error")
    }
}

#[derive(Clone, Copy)]
pub struct NopCache {}

#[async_trait]
impl Cache for NopCache {
    type CacheErrorT = NopCacheError;

    async fn all_exist<CA>(&mut self, _cacheables: &[CA]) -> bool
    where
        CA: Cacheable + Send + Sync + Clone + 'static,
    {
        false
    }

    async fn store<CA>(&mut self, _identity: CA) -> Result<(), Self::CacheErrorT>
    where
        CA: Cacheable + Send + Sync + Clone + 'static,
    {
        tracing::debug!("nopcache.store operation");
        Ok(())
    }

    async fn filter_cached<CA>(&mut self, cacheables: &[CA]) -> Vec<CA>
    where
        CA: Cacheable + Send + Sync + Clone + 'static,
    {
        cacheables.to_vec()
    }
}
