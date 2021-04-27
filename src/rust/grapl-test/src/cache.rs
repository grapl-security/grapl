use async_trait::async_trait;
use sqs_executor::cache::{Cache,
                          Cacheable};

use crate::error::GenericError;

#[derive(Clone)]
pub struct EmptyCache {}

#[async_trait]
impl Cache for EmptyCache {
    type CacheErrorT = GenericError;

    async fn store<CA>(&mut self, _: CA) -> Result<(), Self::CacheErrorT>
    where
        CA: Cacheable + Send + Sync + Clone + 'static,
    {
        Ok(())
    }

    async fn filter_cached<CA>(&mut self, cacheables: &[CA]) -> Vec<CA>
    where
        CA: Cacheable + Send + Sync + Clone + 'static,
    {
        cacheables.into_iter().map(|item| item.clone()).collect()
    }
}

impl EmptyCache {
    pub fn new() -> Self {
        Self {}
    }
}
