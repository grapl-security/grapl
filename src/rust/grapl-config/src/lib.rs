use std::{io::Stdout,
          str::FromStr,
          time::Duration};

use color_eyre::Help;
use grapl_observe::metric_reporter::MetricReporter;
use rusoto_core::{Region,
                  RusotoError};
use rusoto_s3::S3;
use rusoto_sqs::{ListQueuesRequest,
                 Sqs};
use sqs_executor::{make_ten,
                   redis_cache::RedisCache};
use tracing::debug;
use tracing_subscriber::EnvFilter;

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
    pub is_local: bool,
}

pub fn _init_grapl_env(
    service_name: &str,
) -> (ServiceEnv, tracing_appender::non_blocking::WorkerGuard) {
    let env = ServiceEnv {
        service_name: service_name.to_string(),
        is_local: is_local(),
    };
    let tracing_guard = _init_grapl_log(&env);
    tracing::info!(env=?env, "initializing environment");
    (env, tracing_guard)
}

pub fn is_local() -> bool {
    std::env::var("IS_LOCAL")
        .map(|is_local| is_local.to_lowercase() == "true")
        .unwrap_or(false)
}

pub async fn event_cache(env: &ServiceEnv) -> RedisCache {
    let cache_address =
        std::env::var("EVENT_CACHE_CLUSTER_ADDRESS").expect("EVENT_CACHE_CLUSTER_ADDRESS");
    RedisCache::new(
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

pub fn dest_queue_url() -> String {
    std::env::var("DEST_QUEUE_URL").expect("DEST_QUEUE_URL")
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

pub fn _init_grapl_log(env: &ServiceEnv) -> tracing_appender::non_blocking::WorkerGuard {
    let filter = EnvFilter::from_default_env();
    let (non_blocking, guard) = tracing_appender::non_blocking(std::io::stdout());
    if env.is_local {
        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .with_writer(non_blocking)
            .init();
    } else {
        tracing_subscriber::fmt()
            .json()
            .with_env_filter(filter)
            .with_writer(non_blocking)
            .init();
    }
    guard
}

pub fn ux_bucket() -> String {
    std::env::var("UX_BUCKET").expect("UX_BUCKET")
}

pub fn source_queue_url() -> String {
    std::env::var("SOURCE_QUEUE_URL").expect("SOURCE_QUEUE_URL")
}

pub fn dead_letter_queue_url() -> String {
    std::env::var("DEAD_LETTER_QUEUE_URL").expect("DEAD_LETTER_QUEUE_URL")
}

pub fn retry_queue_url() -> String {
    std::env::var("RETRY_QUEUE_URL").expect("RETRY_QUEUE_URL")
}

pub fn mg_alphas() -> Vec<String> {
    return std::env::var("MG_ALPHAS")
        .expect("MG_ALPHAS")
        .split(',')
        .map(|mg| format!("http://{}", mg))
        .collect();
}

pub async fn wait_for_s3(s3_client: impl S3) -> color_eyre::Result<()> {
    wait_loop(150, || async {
        match s3_client.list_buckets().await {
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
        F: std::future::Future<Output=Result<(), E>>,
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

        tokio::time::delay_for(Duration::from_secs(2)).await;
    }

    errs
}

pub fn static_mapping_table_name() -> String {
    std::env::var("STATIC_MAPPING_TABLE").expect("STATIC_MAPPING_TABLE")
}

pub fn dynamic_session_table_name() -> String {
    std::env::var("DYNAMIC_SESSION_TABLE").expect("DYNAMIC_SESSION_TABLE")
}

pub fn process_history_table_name() -> String {
    std::env::var("PROCESS_HISTORY_TABLE").expect("PROCESS_HISTORY_TABLE")
}

pub fn file_history_table_name() -> String {
    std::env::var("FILE_HISTORY_TABLE").expect("FILE_HISTORY_TABLE")
}

pub fn inbound_connection_history_table_name() -> String {
    std::env::var("INBOUND_CONNECTION_HISTORY_TABLE")
        .expect("INBOUND_CONNECTION_HISTORY_TABLE")
}

pub fn outbound_connection_history_table_name() -> String {
    std::env::var("OUTBOUND_CONNECTION_HISTORY_TABLE")
        .expect("OUTBOUND_CONNECTION_HISTORY_TABLE")
}

pub fn network_connection_history_table_name() -> String {
    std::env::var("NETWORK_CONNECTION_HISTORY_TABLE")
        .expect("NETWORK_CONNECTION_HISTORY_TABLE")
}

pub fn ip_connection_history_table_name() -> String {
    std::env::var("IP_CONNECTION_HISTORY_TABLE").expect("IP_CONNECTION_HISTORY_TABLE")
}

pub fn asset_id_mappings_table_name() -> String {
    std::env::var("ASSET_ID_MAPPINGS").expect("ASSET_ID_MAPPINGS")
}
