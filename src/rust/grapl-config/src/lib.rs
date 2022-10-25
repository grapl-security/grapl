use std::str::FromStr;

use rusoto_core::Region;

pub mod env_helpers;
mod postgres;
pub use postgres::{
    PostgresClient,
    PostgresDbInitError,
    PostgresUrl,
    ToPostgresUrl,
};
pub use secrecy::{
    Secret,
    SecretString,
};

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
