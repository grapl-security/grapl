use kafka::config::{
    ConsumerConfig,
    RetryProducerConfig,
};

#[derive(clap::Parser, Debug)]
pub struct GeneratorDispatcherConfig {
    #[clap(flatten)]
    pub kafka_config: ConsumerConfig,

    #[clap(flatten)]
    pub kafka_retry_producer_config: RetryProducerConfig,

    #[clap(long, env = "WORKER_POOL_SIZE")]
    pub worker_pool_size: usize,

    #[clap(long, env = "GENERATOR_IDS_CACHE_CAPACITY")]
    pub generator_ids_cache_capacity: usize,

    #[clap(long, env = "GENERATOR_IDS_CACHE_TTL_MS")]
    pub generator_ids_cache_ttl_ms: u64,

    #[clap(long, env = "GENERATOR_IDS_CACHE_UPDATER_POOL_SIZE")]
    pub generator_ids_cache_updater_pool_size: usize,

    #[clap(long, env = "GENERATOR_IDS_CACHE_UPDATER_QUEUE_DEPTH")]
    pub generator_ids_cache_updater_queue_depth: usize,
}
