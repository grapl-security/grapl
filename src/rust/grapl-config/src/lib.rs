use std::str::FromStr;

use grapl_tracing::{
    setup_tracing,
    WorkerGuard,
};
use rusoto_core::Region;

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
