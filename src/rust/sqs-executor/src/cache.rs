use std::{collections::hash_map::DefaultHasher,
          hash::{Hash,
                 Hasher}};

use async_trait::async_trait;

use crate::errors::{CheckedError,
                    Recoverable};

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

    async fn exists<CA: Cacheable + Send + Sync + Clone + 'static>(
        &mut self,
        cacheable: CA,
    ) -> bool {
        // If cacheable doesn't return from filter_cached then
        // we know it exists in the cache
        self.filter_cached(vec![cacheable]).await.is_empty()
    }

    async fn store(&mut self, identity: Vec<u8>) -> Result<(), Self::CacheErrorT>;

    async fn store_all(&mut self, identities: Vec<Vec<u8>>) -> Result<(), Self::CacheErrorT> {
        for identity in identities.into_iter() {
            self.store(identity).await?;
        }
        Ok(())
    }

    async fn filter_cached<CA: Cacheable + Send + Sync + Clone + 'static>(
        &mut self,
        cacheables: Vec<CA>,
    ) -> Vec<CA>;
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

    async fn exists<CA: Cacheable + Send + Sync + Clone + 'static>(
        &mut self,
        _cacheable: CA,
    ) -> bool {
        false
    }

    async fn store(&mut self, _identity: Vec<u8>) -> Result<(), Self::CacheErrorT> {
        tracing::debug!("nopcache.store operation");
        Ok(())
    }

    async fn filter_cached<CA: Cacheable + Send + Sync + Clone + 'static>(
        &mut self,
        cacheables: Vec<CA>,
    ) -> Vec<CA> {
        cacheables
    }
}
