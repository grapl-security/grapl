use crate::ServiceEnv;
use rusoto_sqs::SqsClient;
use rusoto_s3::{S3Client, S3};
use std::io::Stdout;
use sqs_executor::redis_cache::RedisCache;
use grapl_observe::metric_reporter::MetricReporter;
use sqs_executor::s3_event_emitter::S3EventEmitter;
use sqs_executor::{time_based_key_fn, make_ten};
use rusoto_core::{HttpClient, Region};
use rusoto_dynamodb::DynamoDbClient;
use rusoto_cloudwatch::CloudWatchClient;

#[async_trait::async_trait]
pub trait AsyncFrom<T, S> {
    async fn async_from(t: T) -> S;
}


pub trait FromEnv<S> {
    fn from_env() -> S;
}

impl FromEnv<CloudWatchClient> for CloudWatchClient {
    fn from_env() -> CloudWatchClient {
        let cloudwatch_endpoint = std::env::var("CLOUDWATCH_ENDPOINT").ok();
        let cloudwatch_access_key_id = std::env::var("CLOUDWATCH_ACCESS_KEY_ID").ok();
        let cloudwatch_access_key_secret = std::env::var("CLOUDWATCH_ACCESS_KEY_SECRET").ok();

        match (cloudwatch_endpoint, cloudwatch_access_key_id, cloudwatch_access_key_secret) {
            (Some(cloudwatch_endpoint), Some(cloudwatch_access_key_id), Some(cloudwatch_access_key_secret)) => {
                CloudWatchClient::new_with(
                    HttpClient::new().expect("failed to create request dispatcher"),
                    rusoto_credential::StaticProvider::new_minimal(
                        cloudwatch_access_key_id.to_owned(),
                        cloudwatch_access_key_secret.to_owned(),
                    ),
                    Region::Custom {
                        name: "localcloudwatch".to_string(),
                        endpoint: cloudwatch_endpoint.to_string(),
                    },
                )
            }
            (Some(cloudwatch_endpoint), None, None) => {
                CloudWatchClient::new(Region::Custom {
                    name: "localcloudwatch".to_string(),
                    endpoint: cloudwatch_endpoint.to_string(),
                })
            }
            (None, None, None) => {
                CloudWatchClient::new(crate::region())
            }
            _ => {
                panic!("Must specify cloudwatch_endpoint and/or both of cloudwatch_access_key_id, cloudwatch_access_key_secret")
            }
        }
    }
}

impl FromEnv<DynamoDbClient> for DynamoDbClient {
    fn from_env() -> DynamoDbClient {
        let dynamodb_endpoint = std::env::var("DYNAMODB_ENDPOINT").ok();
        let dynamodb_access_key_id = std::env::var("DYNAMODB_ACCESS_KEY_ID").ok();
        let dynamodb_access_key_secret = std::env::var("DYNAMODB_ACCESS_KEY_SECRET").ok();

        match (dynamodb_endpoint, dynamodb_access_key_id, dynamodb_access_key_secret) {
            (Some(dynamodb_endpoint), Some(dynamodb_access_key_id), Some(dynamodb_access_key_secret)) => {
                DynamoDbClient::new_with(
                    HttpClient::new().expect("failed to create request dispatcher"),
                    rusoto_credential::StaticProvider::new_minimal(
                        dynamodb_access_key_id.to_owned(),
                        dynamodb_access_key_secret.to_owned(),
                    ),
                    Region::Custom {
                        name: "localdynamodb".to_string(),
                        endpoint: dynamodb_endpoint.to_string(),
                    },
                )
            }
            (Some(dynamodb_endpoint), None, None) => {
                DynamoDbClient::new(Region::Custom {
                    name: "localdynamodb".to_string(),
                    endpoint: dynamodb_endpoint.to_string(),
                })
            }
            (None, None, None) => {
                DynamoDbClient::new(crate::region())
            }
            _ => {
                panic!("Must specify dynamodb_endpoint and/or both of dynamodb_access_key_id, dynamodb_access_key_secret")
            }
        }
    }
}

impl FromEnv<SqsClient> for SqsClient {
    fn from_env() -> SqsClient {
        let sqs_endpoint = std::env::var("SQS_ENDPOINT").ok();
        let sqs_access_key_id = std::env::var("SQS_ACCESS_KEY_ID").ok();
        let sqs_access_key_secret = std::env::var("SQS_ACCESS_KEY_SECRET").ok();

        match (sqs_endpoint, sqs_access_key_id, sqs_access_key_secret) {
            (Some(sqs_endpoint), Some(sqs_access_key_id), Some(sqs_access_key_secret)) => {
                SqsClient::new_with(
                    HttpClient::new().expect("failed to create request dispatcher"),
                    rusoto_credential::StaticProvider::new_minimal(
                        sqs_access_key_id.to_owned(),
                        sqs_access_key_secret.to_owned(),
                    ),
                    Region::Custom {
                        name: "localsqs".to_string(),
                        endpoint: sqs_endpoint.to_string(),
                    },
                )
            }
            (Some(sqs_endpoint), None, None) => {
                SqsClient::new(Region::Custom {
                    name: "localsqs".to_string(),
                    endpoint: sqs_endpoint.to_string(),
                })
            }
            (None, None, None) => {
                SqsClient::new(crate::region())
            }
            _ => {
                panic!("Must specify sqs_endpoint and/or both of sqs_access_key_id, sqs_access_key_secret")
            }
        }
    }
}

impl FromEnv<S3Client> for S3Client {
    fn from_env() -> S3Client {
        let s3_endpoint = std::env::var("S3_ENDPOINT").ok();
        let s3_access_key_id = std::env::var("S3_ACCESS_KEY_ID").ok();
        let s3_access_key_secret = std::env::var("S3_ACCESS_KEY_SECRET").ok();

        match (s3_endpoint, s3_access_key_id, s3_access_key_secret) {
            (Some(s3_endpoint), Some(s3_access_key_id), Some(s3_access_key_secret)) => {
                S3Client::new_with(
                    HttpClient::new().expect("failed to create request dispatcher"),
                    rusoto_credential::StaticProvider::new_minimal(
                        s3_access_key_id.to_owned(),
                        s3_access_key_secret.to_owned(),
                    ),
                    Region::Custom {
                        name: "locals3".to_string(),
                        endpoint: s3_endpoint.to_string(),
                    },
                )
            }
            (Some(s3_endpoint), None, None) => {
                S3Client::new(Region::Custom {
                    name: "locals3".to_string(),
                    endpoint: s3_endpoint.to_string(),
                })
            }
            (None, None, None) => {
                S3Client::new(crate::region())
            }
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
        F: Fn(&[u8]) -> String,
{
        S3EventEmitter::new(
            S3Client::from_env(),
            crate::dest_bucket(),
            key_fn,
            MetricReporter::new(&env.service_name),
        )
}

pub async fn s3_event_emitters_from_env<F>(env: &ServiceEnv, key_fn: F) -> [S3EventEmitter<S3Client, F>; 10]
    where
        F: Clone + Fn(&[u8]) -> String,
{
    make_ten(
        async {s3_event_emitter_from_env(env, key_fn)}
    ).await
}
// impl<F> From<(&ServiceEnv, F)> for S3EventEmitter<S3Client, F>
//     where
//         F: Fn(&[u8]) -> String,
// {
//
// }
//
//
//







