use std::{collections::hash_map::DefaultHasher,
          hash::{Hash,
                 Hasher}};

use async_trait::async_trait;

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

#[derive(Clone)]
pub enum CacheResponse {
    Hit,
    Miss,
}

#[async_trait]
pub trait Cache: Clone {
    async fn get<CA: Cacheable + Send + Sync + 'static>(
        &mut self,
        cacheable: CA,
    ) -> Result<CacheResponse, crate::error::Error>;
    async fn store(&mut self, identity: Vec<u8>) -> Result<(), crate::error::Error>;
}

#[async_trait]
pub trait ReadableCache {
    async fn get<CA: Cacheable + Send + Sync + 'static>(
        &mut self,
        cacheable: CA,
    ) -> Result<CacheResponse, crate::error::Error>;
}

#[async_trait]
impl<C> ReadableCache for C
where
    C: Cache + Send + Sync + 'static,
{
    async fn get<CA>(&mut self, cacheable: CA) -> Result<CacheResponse, crate::error::Error>
    where
        CA: Cacheable + Send + Sync + 'static,
    {
        Cache::get(self, cacheable).await
    }
}

#[derive(Clone)]
pub struct NopCache {}

#[async_trait]
impl Cache for NopCache {
    async fn get<CA: Cacheable + Send + Sync + 'static>(
        &mut self,
        _cacheable: CA,
    ) -> Result<CacheResponse, crate::error::Error> {
        Ok(CacheResponse::Miss)
    }
    async fn store(&mut self, _identity: Vec<u8>) -> Result<(), crate::error::Error> {
        Ok(())
    }
}
