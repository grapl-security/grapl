use std::{
    io::Stdout,
    str::FromStr,
    time::Duration,
};

use color_eyre::Help;
use grapl_observe::metric_reporter::MetricReporter;
use opentelemetry::{
    global,
    sdk::propagation::TraceContextPropagator,
};
use rusoto_core::{
    Region,
    RusotoError,
};
use rusoto_sqs::{
    ListQueuesRequest,
    Sqs,
};
use sqs_executor::{
    make_ten,
    redis_cache::RedisCache,
};
use tracing::debug;
use tracing_subscriber::{
    prelude::*,
    EnvFilter,
};

pub mod env_helpers;

#[macro_export]
macro_rules! init_grapl_env {
    () => {
        $crate::_init_grapl_env(&module_path!().replace("-", "_"))
    };
    ($module_name: literal) => {
        $crate::_init_grapl_env($module_name)
    };
}

#[derive(Debug)]
pub struct ServiceEnv {
    pub service_name: String,
}

pub fn _init_grapl_env(
    service_name: &str,
) -> (ServiceEnv, tracing_appender::non_blocking::WorkerGuard) {
    let env = ServiceEnv {
        service_name: service_name.to_string(),
    };
    let tracing_guard = _init_grapl_log(&env.service_name);
    tracing::info!(env=?env, "initializing environment");
    (env, tracing_guard)
}

pub async fn event_cache(env: &ServiceEnv) -> RedisCache {
    let cache_address = std::env::var("REDIS_ENDPOINT").expect("REDIS_ENDPOINT");
    if !cache_address.starts_with("redis://") {
        panic!(
            "Expected redis client with redis://, but got {}",
            cache_address
        );
    }
    let lru_cache_size = std::env::var("LRU_CACHE_SIZE")
        .unwrap_or(String::from("1000000"))
        .parse::<usize>()
        .unwrap_or(1_000_000);
    RedisCache::with_lru_capacity(
        lru_cache_size,
        cache_address.to_owned(),
        MetricReporter::<Stdout>::new(&env.service_name),
    )
    .await
    .expect("Could not create redis client")
}

pub async fn event_caches(env: &ServiceEnv) -> [RedisCache; 10] {
    make_ten(event_cache(env)).await
}

pub fn dest_bucket() -> String {
    std::env::var("DEST_BUCKET_NAME").expect("DEST_BUCKET_NAME")
}

pub fn region() -> Region {
    let region_override_endpoint = std::env::var("AWS_REGION_ENDPOINT_OVERRIDE");

    match region_override_endpoint {
        Ok(region_override_endpoint) => Region::Custom {
            name: "override".to_string(),
            endpoint: region_override_endpoint,
        },
        Err(_) => {
            let region_str = std::env::var("AWS_REGION").expect("AWS_REGION");
            Region::from_str(&region_str).expect("Region error")
        }
    }
}

pub fn _init_grapl_log(service_name: &str) -> tracing_appender::non_blocking::WorkerGuard {
    let filter = EnvFilter::from_default_env();

    let (non_blocking, guard) = tracing_appender::non_blocking(std::io::stdout());

    // init json logging layer
    let log_layer = tracing_subscriber::fmt::layer()
        .json()
        .with_writer(non_blocking);

    // init tracing layer
    global::set_text_map_propagator(TraceContextPropagator::new());
    let tracer = opentelemetry_jaeger::new_pipeline()
        .with_service_name(service_name)
        .install_batch(opentelemetry::runtime::Tokio)
        .expect("opentelemetry-jaeger tracer");

    // register a subscriber with all the layers
    tracing_subscriber::registry()
        .with(filter)
        .with(log_layer)
        .with(tracing_opentelemetry::layer().with_tracer(tracer))
        .init();
    guard
}

pub fn source_queue_url() -> String {
    std::env::var("SOURCE_QUEUE_URL").expect("SOURCE_QUEUE_URL")
}

pub fn dead_letter_queue_url() -> String {
    std::env::var("DEAD_LETTER_QUEUE_URL").expect("DEAD_LETTER_QUEUE_URL")
}

pub fn mg_alphas() -> Vec<String> {
    return std::env::var("MG_ALPHAS")
        .expect("MG_ALPHAS")
        .split(',')
        .map(|mg| {
            if mg.contains("http://") {
                panic!("Expected mg_alphas without http://, but got {}", mg);
            }
            format!("http://{}", mg)
        })
        .collect();
}

pub async fn wait_for_sqs(
    sqs_client: impl Sqs,
    queue_name_prefix: impl Into<String>,
) -> color_eyre::Result<()> {
    let queue_name_prefix = queue_name_prefix.into();
    wait_loop(150, || async {
        match sqs_client
            .list_queues(ListQueuesRequest {
                max_results: None,
                next_token: None,
                queue_name_prefix: Some(queue_name_prefix.clone()),
            })
            .await
        {
            Err(RusotoError::HttpDispatch(e)) => {
                debug!("Waiting for S3 to become available: {:?}", e);
                Err(e)
            }
            _ => Ok(()),
        }
    })
    .await?;

    Ok(())
}

async fn wait_loop<F, E>(max_tries: u32, f: impl Fn() -> F) -> color_eyre::Result<()>
where
    F: std::future::Future<Output = Result<(), E>>,
    E: std::error::Error + Send + Sync + 'static,
{
    let mut errs: Result<(), _> = Err(eyre::eyre!("wait_loop failed"));
    for _ in 0..max_tries {
        match (f)().await {
            Ok(()) => return Ok(()),
            Err(e) => {
                errs = errs.error(e);
            }
        };

        tokio::time::sleep(Duration::from_secs(2)).await;
    }

    errs
}

pub fn static_mapping_table_name() -> String {
    std::env::var("GRAPL_STATIC_MAPPING_TABLE").expect("GRAPL_STATIC_MAPPING_TABLE")
}

pub fn dynamic_session_table_name() -> String {
    std::env::var("GRAPL_DYNAMIC_SESSION_TABLE").expect("GRAPL_DYNAMIC_SESSION_TABLE")
}

pub fn source_compression() -> String {
    std::env::var("SOURCE_COMPRESSION").unwrap_or(String::from("Zstd"))
}
