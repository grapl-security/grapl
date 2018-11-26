use sha2::{Sha224, Digest};

use std::sync::Mutex;
use std::sync::Arc;
use lru_time_cache::LruCache;

use failure::Error;
use std::time::Duration;
use session::Session;

pub trait IdentityKeyable {
    fn get_cache_key(&self, pepper: &[u8]) -> Vec<u8>;
    fn get_future_cache_key(&self, pepper: &[u8]) -> Vec<u8>;
}

impl IdentityKeyable {

}


impl<T> IdentityKeyable for T
    where T: Session

{


    fn get_cache_key(&self, pepper: &[u8]) -> Vec<u8> {

        println!("let timestamp ");
        let timestamp = (self.get_timestamp() - self.get_timestamp() % 10).to_string();
        println!("let table_name ");
        let table_name = self.get_table_name();
        println!("let session_key ");
        let session_key = self.get_key();
        println!("let asset_id ");
        let asset_id = self.get_asset_id().as_bytes();

        let mut hasher = Sha224::new();

        hasher.input(table_name.as_bytes());
        hasher.input(session_key.as_bytes());
        hasher.input(asset_id);
        hasher.input(timestamp.as_bytes());
        hasher.input(pepper);

        let key= hasher.result();

        eprintln!("key.to_vec()");
        key.to_vec()
    }

    fn get_future_cache_key(&self, pepper: &[u8]) -> Vec<u8> {
        let timestamp = (self.get_timestamp() - self.get_timestamp() % 10 + 10).to_string();
        let table_name = self.get_table_name();
        let session_key = self.get_key();
        let asset_id = self.get_asset_id().as_bytes();

        let mut hasher = Sha224::new();

        hasher.input(table_name.as_bytes());
        hasher.input(session_key.as_bytes());
        hasher.input(asset_id);
        hasher.input(timestamp.as_bytes());
        hasher.input(pepper);

        let key= hasher.result();

        key.to_vec()
    }
}


#[derive(Clone)]
pub struct IdentityCache<'a> {
    lru: Arc<Mutex<LruCache<Vec<u8>, String>>>,
    pepper: &'a [u8]
}

impl<'a> IdentityCache<'a> {
    pub fn new(max_count: usize, time_to_live: Duration, pepper: &'a [u8]) -> IdentityCache<'a> {
        IdentityCache {
            lru: Arc::new(
                Mutex::new(
                    LruCache::with_expiry_duration_and_capacity(time_to_live, max_count)
                )
            ),
            pepper
        }
    }

    pub fn check_cache(&self, session: impl IdentityKeyable) -> Result<Option<String>, Error> {
        println!("fn check_cache(&self");
        let mut lru_cache = self.lru.lock().unwrap();

        let key = session.get_cache_key(&self.pepper);

        let res = Ok(lru_cache.get(&key).map(String::to_owned));

        // If we get a hit, preemptively update cache for the future
        if let Ok(Some(ref id)) = res {
            println!("Cache hit");
            self.preload_cache(session, id);
        }
        res
    }

    pub fn update_cache(&self, session: impl IdentityKeyable, id: impl Into<String>) -> Result<(), Error> {
        println!("fn update_cache(&self, session: impl IdentityKeyable, ");
        let key = session.get_cache_key(&self.pepper);

        self.insert(key, id);

        Ok(())
    }

    fn preload_cache(&self, session: impl IdentityKeyable, id: &str) -> Result<(), Error> {
        println!("preload_cache");
        let future_key = session.get_future_cache_key(self.pepper);

        self.insert(future_key, id.to_owned());
        Ok(())
    }

    fn insert(&self, key: Vec<u8>, id: impl Into<String>) {
        println!("insert(&self");
        let mut lru_cache = self.lru.lock().unwrap();
        println!("Updating cache");
        lru_cache.insert(key, id.into());
    }

}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    use session::Action;
    use std::borrow::Cow;

    #[derive(Debug)]
    struct MockKeyable {
        pub timestamp: u64,
        pub key: &'static [u8]
    }

    impl<'a> IdentityKeyable for &'a MockKeyable {
        fn get_cache_key(&self, pepper: &[u8]) -> Vec<u8> {
            self.get_cache_key(pepper)
        }
        fn get_future_cache_key(&self, pepper: &[u8]) -> Vec<u8> {self.get_future_cache_key(pepper)}
    }

    impl IdentityKeyable for MockKeyable {
        fn get_cache_key(&self, pepper: &[u8]) -> Vec<u8> {
            let timestamp = (self.timestamp - self.timestamp % 10).to_string();

            let mut hasher = Sha224::new();

            hasher.input(self.key);
            hasher.input(timestamp.as_bytes());
            hasher.input(pepper);

            let key = hasher.result();

            key.to_vec()
        }
        fn get_future_cache_key(&self, pepper: &[u8]) -> Vec<u8> {
            let timestamp = (self.timestamp - self.timestamp % 10 + 10).to_string();

            let mut hasher = Sha224::new();

            hasher.input(self.key);
            hasher.input(timestamp.as_bytes());
            hasher.input(pepper);

            let key = hasher.result();

            key.to_vec()
        }
    }


    #[test]
    fn test_set_get_eq() {
        let mut cache = IdentityCache::new(
            10,
            Duration::from_secs(1 << 8),
            b"pepper"
        );

        let identity = "identity";

        let keyable = MockKeyable {
            timestamp: 1234,
            key: &b"key"[..],
        };

        cache.update_cache(&keyable, identity);
//        cache.check_cache(keyable);
    }


    #[test]
    fn test_set_get_skew() {}

    #[test]
    fn test_set_get_preload() {
        // This assert would fire and test will fail.
        // Please note, that private functions can be tested too!
//        assert_eq!(bad_add(1, 2), 3);
    }
}