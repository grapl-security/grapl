use sqs_lambda::redis_cache::RedisCache;
use rusoto_core::Region;
use std::str::FromStr;

pub async fn event_cache() -> RedisCache {
    let cache_address = {
        let generic_event_cache_addr = std::env::var("EVENT_CACHE_ADDR").expect("EVENT_CACHE_ADDR");
        let generic_event_cache_port = std::env::var("EVENT_CACHE_PORT").expect("EVENT_CACHE_PORT");

        format!(
            "{}:{}",
            generic_event_cache_addr,
            generic_event_cache_port,
        )
    };

    RedisCache::new(cache_address.to_owned()).await.expect("Could not create redis client")
}

pub fn region() -> Region {
    let region_override = std::env::var("AWS_REGION_OVERRIDE");

    match region_override {
        Ok(region) => Region::Custom {
            name: "override".to_string(),
            endpoint: region,
        },
        Err(_) => {
            let region_str = std::env::var("AWS_REGION").expect("AWS_REGION");
            Region::from_str(&region_str).expect("Region error")
        }
    }
}
