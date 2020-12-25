use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use async_trait::async_trait;

use crate::errors::CheckedError;

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
    async fn get<
        CA: Cacheable + Send + Sync + 'static,
        CacheErrorT: CheckedError + Send + Sync + 'static,
    >(
        &mut self,
        cacheable: CA,
    ) -> Result<CacheResponse, CacheErrorT>;
    async fn store<
        CacheErrorT: CheckedError + Send + Sync + 'static,
    >(&mut self, identity: Vec<u8>) -> Result<(), CacheErrorT>;
}

#[async_trait]
pub trait ReadableCache {
    async fn get<
        CA: Cacheable + Send + Sync + 'static,
        CacheErrorT: CheckedError + Send + Sync + 'static,
    >(
        &mut self,
        cacheable: CA,
    ) -> Result<CacheResponse, CacheErrorT>;
}

#[derive(Clone)]
pub struct NopCache {}

#[async_trait]
impl Cache for NopCache {
    async fn get<
        CA: Cacheable + Send + Sync + 'static,
        CacheErrorT: CheckedError + Send + Sync + 'static,
    >(
        &mut self,
        _cacheable: CA,
    ) -> Result<CacheResponse, CacheErrorT> {
        Ok(CacheResponse::Miss)
    }
    async fn store<
        CacheErrorT: CheckedError + Send + Sync + 'static,
    >(&mut self, _identity: Vec<u8>) -> Result<(), CacheErrorT> {
        Ok(())
    }
}
