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

    let bucket_prefix = std::env::var("BUCKET_PREFIX").expect("BUCKET_PREFIX");
    let destination_bucket = format!("{}-unid-subgraphs-generated-bucket", bucket_prefix);

    let sqs_client = SqsClient::new(grapl_config::region());
    let s3_client = S3Client::new(grapl_config::region());

    let cache_address = {
        let generic_event_cache_addr =
            std::env::var("EVENT_CACHE_ADDR").expect("GENERIC_EVENT_CACHE_ADDR");
        let generic_event_cache_port =
            std::env::var("EVENT_CACHE_PORT").expect("GENERIC_EVENT_CACHE_PORT");

        format!("{}:{}", generic_event_cache_addr, generic_event_cache_port,)
    };

    let cache = &mut make_ten(async {
        RedisCache::new(
            cache_address.to_owned(),
            MetricReporter::<Stdout>::new(&env.service_name),
        )
        .await
        .expect("Could not create redis client")
    })
    .await;

    // ;
    // let cache = &mut [rs.clone(), rs.clone(), rs.clone(), rs.clone(), rs.clone(), rs.clone(), rs.clone(), rs.clone(), rs.clone(), rs];

    let sysmon_subgraph_generator = &mut make_ten(async {
        SysmonSubgraphGenerator::new(
            NopCache {},
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
        std::env::var("QUEUE_URL").expect("QUEUE_URL"),
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
