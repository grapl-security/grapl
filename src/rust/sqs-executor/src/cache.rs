use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use async_trait::async_trait;

use crate::errors::{CheckedError, Recoverable};

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
    type CacheErrorT: CheckedError + Send + Sync + 'static;
    async fn get<CA: Cacheable + Send + Sync + 'static>(
        &mut self,
        cacheable: CA,
    ) -> Result<CacheResponse, Self::CacheErrorT>;
    async fn store(&mut self, identity: Vec<u8>) -> Result<(), Self::CacheErrorT>;
}

// #[async_trait]
// pub trait ReadableCache {
//     async fn get<
//         CA: Cacheable + Send + Sync + 'static,
//     >(
//         &mut self,
//         cacheable: CA,
//     ) -> Result<CacheResponse, CacheErrorT>;
// }

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

    async fn get<CA: Cacheable + Send + Sync + 'static>(
        &mut self,
        _cacheable: CA,
    ) -> Result<CacheResponse, Self::CacheErrorT> {
        Ok(CacheResponse::Miss)
    }
    async fn store(&mut self, _identity: Vec<u8>) -> Result<(), Self::CacheErrorT> {
        Ok(())
    }
}
