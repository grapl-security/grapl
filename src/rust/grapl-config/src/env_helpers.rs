use crate::ServiceEnv;
use rusoto_sqs::SqsClient;
use rusoto_s3::{S3Client, S3};
use std::io::Stdout;
use sqs_executor::redis_cache::RedisCache;
use grapl_observe::metric_reporter::MetricReporter;
use sqs_executor::s3_event_emitter::S3EventEmitter;
use sqs_executor::{time_based_key_fn, make_ten};

#[async_trait::async_trait]
pub trait AsyncFrom<T, S> {
    async fn async_from(t: T) -> S;
}


pub trait FromEnv<S> {
    fn from_env() -> S;
}

impl FromEnv<SqsClient> for SqsClient {
    fn from_env() -> SqsClient {
        SqsClient::new(crate::region())
    }
}

impl FromEnv<S3Client> for S3Client {
    fn from_env() -> S3Client {
        S3Client::new(crate::region())
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







