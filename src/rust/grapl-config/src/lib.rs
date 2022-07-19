use std::{
    str::FromStr,
    time::Duration,
};

use color_eyre::Help;
use grapl_tracing::{
    setup_tracing,
    WorkerGuard,
};
use rusoto_core::{
    Region,
    RusotoError,
};
use rusoto_sqs::{
    ListQueuesRequest,
    Sqs,
};
use tracing::debug;

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

pub fn _init_grapl_env(service_name: &str) -> (ServiceEnv, WorkerGuard) {
    let env = ServiceEnv {
        service_name: service_name.to_string(),
    };
    let tracing_guard = _init_grapl_log(&env.service_name);
    tracing::info!(env=?env, "initializing environment");
    (env, tracing_guard)
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

pub fn _init_grapl_log(service_name: &str) -> WorkerGuard {
    // This should be deprecated and moved to a direct `setup_tracing` call
    setup_tracing(service_name).expect("Setting up tracing")
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
