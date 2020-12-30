#![type_length_limit = "1334469"]

use std::io::Stdout;
use std::str::FromStr;
use std::time::Duration;

use chrono::Utc;
use log::*;
use rusoto_core::Region;
use rusoto_sqs::SqsClient;

use grapl_config::*;
use grapl_observe::metric_reporter::MetricReporter;
use sqs_executor::cache::NopCache;
use sqs_executor::event_retriever::S3PayloadRetriever;
use sqs_executor::redis_cache::RedisCache;
use sqs_executor::s3_event_emitter::S3EventEmitter;
use sqs_executor::{make_ten, time_based_key_fn};

use crate::generator::SysmonSubgraphGenerator;
use crate::metrics::SysmonSubgraphGeneratorMetrics;
use crate::serialization::{SubgraphSerializer, ZstdDecoder};
use grapl_config::env_helpers::FromEnv;
use rusoto_s3::S3Client;
use std::convert::TryInto;

mod generator;
mod metrics;
mod models;
mod serialization;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let env = grapl_config::init_grapl_env!();
    info!("Starting sysmon-subgraph-generator 4");

    let destination_bucket = std::env::var("DEST_BUCKET_NAME").expect("DEST_BUCKET_NAME");

    let sqs_client = SqsClient::from_env();
    let s3_client = S3Client::from_env();

    let cache = &mut event_caches(&env).await;

    let sysmon_subgraph_generator = &mut make_ten(async {
        SysmonSubgraphGenerator::new(
            cache[0].clone(),
            SysmonSubgraphGeneratorMetrics::new(&env.service_name),
        )
    })
    .await;
    let serializer = &mut make_ten(async { SubgraphSerializer::default() }).await;

    let s3_emitter = &mut make_ten(async {
        S3EventEmitter::new(
            s3_client.clone(),
            destination_bucket.clone(),
            time_based_key_fn,
            MetricReporter::new(&env.service_name),
        )
    })
    .await;

    let s3_payload_retriever = &mut make_ten(async {
        S3PayloadRetriever::new(
            |region_str| S3Client::new(Region::from_str(&region_str).expect("region_str")),
            ZstdDecoder::default(),
            MetricReporter::new(&env.service_name),
        )
    })
    .await;

    info!("Starting process_loop");
    sqs_executor::process_loop(
        std::env::var("SOURCE_QUEUE_URL").expect("SOURCE_QUEUE_URL"),
        std::env::var("DEAD_LETTER_QUEUE_URL").expect("DEAD_LETTER_QUEUE_URL"),
        cache,
        sqs_client.clone(),
        sysmon_subgraph_generator,
        s3_payload_retriever,
        s3_emitter,
        serializer,
        MetricReporter::new(&env.service_name),
    )
    .await;

    Ok(())
}
