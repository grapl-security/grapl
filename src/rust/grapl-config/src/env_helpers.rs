use std::{
    io::Stdout,
    str::FromStr,
};

use grapl_observe::metric_reporter::MetricReporter;
use rusoto_cloudwatch::CloudWatchClient;
use rusoto_core::{
    HttpClient,
    Region,
};
use rusoto_dynamodb::DynamoDbClient;
use rusoto_s3::S3Client;
use rusoto_sqs::SqsClient;
use sqs_executor::{
    make_ten,
    s3_event_emitter::S3EventEmitter,
};

use crate::ServiceEnv;

const ENV_ENDPOINT: &'static str = "AWS_ENDPOINT";
const ENV_ACCESS_KEY_ID: &'static str = "AWS_ACCESS_KEY_ID";
const ENV_ACCESS_KEY_SECRET: &'static str = "AWS_ACCESS_KEY_SECRET";

#[async_trait::async_trait]
pub trait AsyncFrom<T, S> {
    async fn async_from(t: T) -> S;
}

pub trait FromEnv<S> {
    fn from_env() -> S;
}

impl FromEnv<CloudWatchClient> for CloudWatchClient {
    fn from_env() -> CloudWatchClient {
        let cloudwatch_endpoint = std::env::var(ENV_ENDPOINT).ok();
        let cloudwatch_access_key_id = std::env::var(ENV_ACCESS_KEY_ID).ok();
        let cloudwatch_access_key_secret = std::env::var(ENV_ACCESS_KEY_SECRET).ok();
        let region_name = std::env::var("AWS_REGION").unwrap_or_else(|_| "us-east-1".to_owned());
        match (
            cloudwatch_endpoint,
            cloudwatch_access_key_id,
            cloudwatch_access_key_secret,
        ) {
            (
                Some(cloudwatch_endpoint),
                Some(cloudwatch_access_key_id),
                Some(cloudwatch_access_key_secret),
            ) => CloudWatchClient::new_with(
                HttpClient::new().expect("failed to create request dispatcher"),
                rusoto_credential::StaticProvider::new_minimal(
                    cloudwatch_access_key_id,
                    cloudwatch_access_key_secret,
                ),
                Region::Custom {
                    name: region_name,
                    endpoint: cloudwatch_endpoint,
                },
            ),
            (Some(cloudwatch_endpoint), None, None) => CloudWatchClient::new(Region::Custom {
                name: region_name,
                endpoint: cloudwatch_endpoint,
            }),
            (None, None, None) => CloudWatchClient::new(crate::region()),
            _ => {
                panic!("Must specify cloudwatch_endpoint and/or both of cloudwatch_access_key_id, cloudwatch_access_key_secret")
            }
        }
    }
}

impl FromEnv<DynamoDbClient> for DynamoDbClient {
    fn from_env() -> DynamoDbClient {
        let dynamodb_endpoint = std::env::var(ENV_ENDPOINT).ok();
        let dynamodb_access_key_id = std::env::var(ENV_ACCESS_KEY_ID).ok();
        let dynamodb_access_key_secret = std::env::var(ENV_ACCESS_KEY_SECRET).ok();
        let region_name = std::env::var("AWS_REGION").unwrap_or_else(|_| "us-east-1".to_string());

        match (
            dynamodb_endpoint,
            dynamodb_access_key_id,
            dynamodb_access_key_secret,
        ) {
            (
                Some(dynamodb_endpoint),
                Some(dynamodb_access_key_id),
                Some(dynamodb_access_key_secret),
            ) => DynamoDbClient::new_with(
                HttpClient::new().expect("failed to create request dispatcher"),
                rusoto_credential::StaticProvider::new_minimal(
                    dynamodb_access_key_id,
                    dynamodb_access_key_secret,
                ),
                Region::Custom {
                    name: region_name,
                    endpoint: dynamodb_endpoint,
                },
            ),
            (Some(dynamodb_endpoint), None, None) => DynamoDbClient::new(Region::Custom {
                name: region_name,
                endpoint: dynamodb_endpoint,
            }),
            (None, None, None) => DynamoDbClient::new(crate::region()),
            _ => {
                panic!("Must specify dynamodb_endpoint and/or both of dynamodb_access_key_id, dynamodb_access_key_secret")
            }
        }
    }
}

impl FromEnv<SqsClient> for SqsClient {
    fn from_env() -> SqsClient {
        let sqs_endpoint = std::env::var(ENV_ENDPOINT).ok();
        let sqs_access_key_id = std::env::var(ENV_ACCESS_KEY_ID).ok();
        let sqs_access_key_secret = std::env::var(ENV_ACCESS_KEY_SECRET).ok();
        let region_name = std::env::var("AWS_REGION").unwrap_or_else(|_| "us-east-1".to_string());

        tracing::info!("overriding sqs_endpoint: {:?}", sqs_endpoint);
        tracing::info!("overriding sqs_access_key_id: {:?}", sqs_access_key_id);
        tracing::info!(
            "overriding sqs_access_key_secret: {:?}",
            sqs_access_key_secret
        );

        match (sqs_endpoint, sqs_access_key_id, sqs_access_key_secret) {
            (Some(sqs_endpoint), Some(sqs_access_key_id), Some(sqs_access_key_secret)) => {
                SqsClient::new_with(
                    HttpClient::new().expect("failed to create request dispatcher"),
                    rusoto_credential::StaticProvider::new_minimal(
                        sqs_access_key_id,
                        sqs_access_key_secret,
                    ),
                    Region::Custom {
                        name: region_name,
                        endpoint: sqs_endpoint,
                    },
                )
            }
            (Some(sqs_endpoint), None, None) => SqsClient::new(Region::Custom {
                name: region_name,
                endpoint: sqs_endpoint,
            }),
            (None, None, None) => SqsClient::new(crate::region()),
            _ => {
                panic!("Must specify sqs_endpoint and/or both of sqs_access_key_id, sqs_access_key_secret")
            }
        }
    }
}

#[tracing::instrument]
pub fn init_s3_client(region_name: &str) -> S3Client {
    let region = match std::env::var(ENV_ENDPOINT).ok() {
        Some(custom_endpoint) => Region::Custom {
            name: region_name.to_owned(),
            endpoint: custom_endpoint,
        },
        None => Region::from_str(&region_name)
            .unwrap_or_else(|e| panic!("Invalid region name: {:?} {:?}", region_name, e)),
    };

    let s3_access_key_id = std::env::var(ENV_ACCESS_KEY_ID).ok();
    let s3_access_key_secret = std::env::var(ENV_ACCESS_KEY_SECRET).ok();

    match (s3_access_key_id, s3_access_key_secret) {
        (Some(s3_access_key_id), Some(s3_access_key_secret)) => {
            tracing::debug!(
                "init_s3_client. - overriding s3_access_key_id: {:?}",
                s3_access_key_id
            );
            tracing::debug!(
                "init_s3_client. - overriding s3_access_key_secret: {:?}",
                s3_access_key_secret
            );
            tracing::debug!("init_s3_client. - overriding region_name: {:?}", region);
            S3Client::new_with(
                HttpClient::new().expect("failed to create request dispatcher"),
                rusoto_credential::StaticProvider::new_minimal(
                    s3_access_key_id,
                    s3_access_key_secret,
                ),
                region,
            )
        }
        (None, None) => {
            tracing::debug!("init_s3_client - custom region: {:?}", region);
            S3Client::new(region)
        }
        (_, _) => {
            panic!("Must specify no overrides, or both of s3_access_key_id, s3_access_key_secret")
        }
    }
}

impl FromEnv<S3Client> for S3Client {
    fn from_env() -> S3Client {
        let s3_endpoint = std::env::var(ENV_ENDPOINT).ok();
        let s3_access_key_id = std::env::var(ENV_ACCESS_KEY_ID).ok();
        let s3_access_key_secret = std::env::var(ENV_ACCESS_KEY_SECRET).ok();
        let region_name = std::env::var("AWS_REGION").unwrap_or_else(|_| "us-east-1".to_string());

        tracing::debug!("overriding s3_endpoint: {:?}", s3_endpoint);
        tracing::debug!("overriding s3_access_key_id: {:?}", s3_access_key_id);
        tracing::debug!(
            "overriding s3_access_key_secret: {:?}",
            s3_access_key_secret
        );
        tracing::debug!("overriding region_name: {:?}", region_name);

        match (s3_endpoint, s3_access_key_id, s3_access_key_secret) {
            (Some(s3_endpoint), Some(s3_access_key_id), Some(s3_access_key_secret)) => {
                S3Client::new_with(
                    HttpClient::new().expect("failed to create request dispatcher"),
                    rusoto_credential::StaticProvider::new_minimal(
                        s3_access_key_id,
                        s3_access_key_secret,
                    ),
                    Region::Custom {
                        name: region_name,
                        endpoint: s3_endpoint,
                    },
                )
            }
            (Some(s3_endpoint), None, None) => S3Client::new(Region::Custom {
                name: region_name,
                endpoint: s3_endpoint,
            }),
            (None, None, None) => S3Client::new(crate::region()),
            _ => {
                panic!("Must specify no overrides, or s3_endpoint and/or both of s3_access_key_id, s3_access_key_secret")
            }
        }
    }
}

impl From<&ServiceEnv> for MetricReporter<Stdout> {
    fn from(env: &ServiceEnv) -> Self {
        MetricReporter::new(&env.service_name)
    }
}

pub fn s3_event_emitter_from_env<F>(env: &ServiceEnv, key_fn: F) -> S3EventEmitter<S3Client, F>
where
    F: Clone + Fn(&[u8]) -> String + Send + Sync + 'static,
{
    S3EventEmitter::new(
        S3Client::from_env(),
        crate::dest_bucket(),
        key_fn,
        MetricReporter::new(&env.service_name),
    )
}

pub async fn s3_event_emitters_from_env<F>(
    env: &ServiceEnv,
    key_fn: F,
) -> [S3EventEmitter<S3Client, F>; 10]
where
    F: Clone + Fn(&[u8]) -> String + Send + Sync + 'static,
{
    make_ten(async { s3_event_emitter_from_env(env, key_fn) }).await
}
