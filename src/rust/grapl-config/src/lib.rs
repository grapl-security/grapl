use std::str::FromStr;
use tracing_subscriber::EnvFilter;

use color_eyre::Help;
use rusoto_core::{Region, RusotoError};
use rusoto_s3::S3;
use rusoto_sqs::{ListQueuesRequest, Sqs};
use sqs_lambda::redis_cache::RedisCache;
use std::time::Duration;
use tracing::debug;

#[macro_export]
macro_rules! init_grapl_env {
    () => {
        $crate::_init_grapl_env(&module_path!().replace("-", "_"))
    };
    ($module_name: literal) => {
        $crate::_init_grapl_env($module_name)
    };
}

pub struct ServiceEnv {
    pub service_name: String,
    pub is_local: bool,
}

pub fn _init_grapl_env(service_name: &str) -> ServiceEnv {
    let env = ServiceEnv {
        service_name: service_name.to_string(),
        is_local: is_local(),
    };
    _init_grapl_log(&env);
    env
}

fn is_local() -> bool {
    std::env::var("IS_LOCAL")
        .map(|is_local| is_local.to_lowercase().parse().unwrap_or(false))
        .unwrap_or(false)
}

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

pub fn _init_grapl_log(env: &ServiceEnv) {
    let filter = EnvFilter::from_default_env()
        .add_directive("warn".parse().expect("Invalid directive"))
        .add_directive(
            format!("{}={}", env.service_name, grapl_log_level())
                .parse()
                .expect("Invalid directive"),
        );
    if env.is_local {
        tracing_subscriber::fmt().with_env_filter(filter).init();
    } else {
        tracing_subscriber::fmt()
            .json()
            .with_env_filter(filter)
            .init();
    }
}

pub fn mg_alphas() -> Vec<String> {
    return std::env::var("MG_ALPHAS")
        .expect("MG_ALPHAS")
        .split(',')
        .map(str::to_string)
        .collect();
}

pub fn parse_host_port(mg_alpha: String) -> (String, u16) {
    let mut splat = mg_alpha.split(":");
    let host = splat.next().expect("missing host").to_owned();
    let port_str = splat.next();
    let port = port_str
        .expect("missing port")
        .parse()
        .expect(&format!("invalid port: \"{:?}\"", port_str));

    (host, port)
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

        tokio::time::delay_for(Duration::from_secs(2)).await;
    }

    errs
}

pub fn static_mapping_table_name() -> String {
    return std::env::var("STATIC_MAPPING_TABLE").expect("STATIC_MAPPING_TABLE");
}

pub fn dynamic_session_table_name() -> String {
    return std::env::var("DYNAMIC_SESSION_TABLE").expect("DYNAMIC_SESSION_TABLE");
}

pub fn process_history_table_name() -> String {
    return std::env::var("PROCESS_HISTORY_TABLE").expect("PROCESS_HISTORY_TABLE");
}

pub fn file_history_table_name() -> String {
    return std::env::var("FILE_HISTORY_TABLE").expect("FILE_HISTORY_TABLE");
}

pub fn inbound_connection_history_table_name() -> String {
    return std::env::var("INBOUND_CONNECTION_HISTORY_TABLE")
        .expect("INBOUND_CONNECTION_HISTORY_TABLE");
}

pub fn outbound_connection_history_table_name() -> String {
    return std::env::var("OUTBOUND_CONNECTION_HISTORY_TABLE")
        .expect("OUTBOUND_CONNECTION_HISTORY_TABLE");
}

pub fn network_connection_history_table_name() -> String {
    return std::env::var("NETWORK_CONNECTION_HISTORY_TABLE")
        .expect("NETWORK_CONNECTION_HISTORY_TABLE");
}

pub fn ip_connection_history_table_name() -> String {
    return std::env::var("IP_CONNECTION_HISTORY_TABLE").expect("IP_CONNECTION_HISTORY_TABLE");
}

pub fn asset_id_mappings_table_name() -> String {
    return std::env::var("ASSET_ID_MAPPINGS").expect("ASSET_ID_MAPPINGS");
}
