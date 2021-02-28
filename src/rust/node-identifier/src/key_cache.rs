use darkredis::{ConnectionPool, Error as RedisError, MSetBuilder};
use std::collections::HashMap;
use tokio::time::Elapsed;
use lazy_static::lazy_static;
use grapl_utils::future_ext::GraplFutureExt;
use std::string::FromUtf8Error;

lazy_static! {
    /// Timeout for requests to Redis
    static ref REDIS_REQUEST_TIMEOUT: std::time::Duration = std::time::Duration::from_millis(300);
    /// Expiration value for set Redis keys (after 30 minutes, the key is unset)
    static ref REDIS_SET_EXPIRATION: std::time::Duration = std::time::Duration::from_secs(30 * 60); // 30 minutes
}

#[derive(thiserror::Error, Debug)]
pub enum IdentityCacheError {
    #[error("RedisError")]
    RedisError(#[from] RedisError),
    #[error("Elapsed")]
    Elapsed(#[from] Elapsed),
    #[error("FromUtf8Error")]
    FromUtf8Error(#[from] FromUtf8Error)
}

pub struct IdentityCache {
    client_pool: ConnectionPool,
}

impl IdentityCache {
    pub async fn resolve_keys(&self, keys: &[String]) -> Result<HashMap<String, String>, IdentityCacheError> {
        let mut client = self.client_pool.get().await;
        let mut cached_keys = HashMap::new();
        let responses: Vec<std::option::Option<Vec<u8>>> = client
            .mget(&keys)
            .timeout(REDIS_REQUEST_TIMEOUT.clone())
            .await??;

        for (key, response) in keys.into_iter().zip(responses.into_iter()) {
            if let Some(response) = response {
                let response = String::from_utf8(response)?;
                cached_keys.insert(key.to_owned(), response);
            }
        }
        Ok(cached_keys)
    }
    
    pub async fn store_keys(&self, identities: &HashMap<String, String>) -> Result<(), IdentityCacheError> {
        let mut client = self.client_pool.get().await;
        let mut mset_builder = MSetBuilder::new();

        for (key, identity) in identities.iter() {
            mset_builder = mset_builder.set(key, identity);
        }

        client.mset(mset_builder).timeout(REDIS_REQUEST_TIMEOUT.clone()).await??;

        Ok(())
    }
}
