use std::str::FromStr;

use rusoto_core::Region;
use sqs_lambda::redis_cache::RedisCache;

pub async fn event_cache() -> RedisCache {
    let cache_address = {
        let generic_event_cache_addr =
            std::env::var("EVENT_CACHE_ADDR").expect("GENERIC_EVENT_CACHE_ADDR");
        let generic_event_cache_port =
            std::env::var("EVENT_CACHE_PORT").expect("GENERIC_EVENT_CACHE_PORT");

        format!("{}:{}", generic_event_cache_addr, generic_event_cache_port,)
    };

    RedisCache::new(cache_address.to_owned())
        .await
        .expect("Could not create redis client")
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

pub fn grapl_log_level() -> log::Level {
    match std::env::var("GRAPL_LOG_LEVEL") {
        Ok(level) => {
            log::Level::from_str(&level).expect(&format!("Invalid logging level {}", level))
        }
        Err(_) => log::Level::Error,
    }
}
