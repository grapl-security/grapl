#![cfg(feature = "integration")]

use grapl_observe::metric_reporter::MetricReporter;
use sqs_executor::cache::Cache;

#[tokio::test]
async fn redis_cache() {
    const LRU_SIZE: usize = 5;
    const TOTAL_SIZE: usize = 10;

    // Create a set of cacheables that we'll store in the cache
    let all_cacheables: Vec<String> = (0..TOTAL_SIZE)
        .map(|i| format!("sqs-executor::tests::redis_cache-{}", i))
        .collect();
    let all_cacheables = all_cacheables.as_slice();
    let (stored, not_stored) = all_cacheables.split_at(LRU_SIZE);

    let redis_endpoint = std::env::var("REDIS_ENDPOINT").expect("REDIS_ENDPOINT");

    let mut cache = sqs_executor::redis_cache::RedisCache::with_lru_capacity(
        LRU_SIZE,
        redis_endpoint,
        MetricReporter::<std::io::Stdout>::new("redis_cache"),
    )
    .await
    .expect("redis client");

    cache.store_all(stored).await.unwrap();

    // When calling `filter_cached` on what we've stored, we expect an empty response.
    assert_eq!(cache.filter_cached(stored).await, Vec::new() as Vec<String>);
    assert_eq!(cache.filter_cached(not_stored).await, not_stored);
    assert_eq!(cache.filter_cached(all_cacheables).await, not_stored);

    assert!(cache.all_exist(stored).await);
    assert!(!cache.all_exist(not_stored).await);
    assert!(!cache.all_exist(all_cacheables).await);

    assert!(!cache.all_exist(&["non-existent-key"]).await);
}
