#![type_length_limit = "1334469"]

use std::io::Stdout;
use std::str::FromStr;
use std::time::Duration;

use chrono::Utc;
use log::*;
use rusoto_core::Region;
use rusoto_sqs::SqsClient;

use graph_generator_lib::*;
use grapl_config::*;
use grapl_observe::metric_reporter::MetricReporter;
use sqs_executor::cache::NopCache;
use sqs_executor::event_retriever::S3PayloadRetriever;
use sqs_executor::redis_cache::RedisCache;
use sqs_executor::s3_event_emitter::S3EventEmitter;
use sqs_executor::time_based_key_fn;

use crate::generator::SysmonSubgraphGenerator;
use crate::metrics::SysmonSubgraphGeneratorMetrics;
use crate::serialization::{ZstdDecoder, SubgraphSerializer};
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

    // let cache_address = {
    //     let generic_event_cache_addr =
    //         std::env::var("EVENT_CACHE_ADDR").expect("GENERIC_EVENT_CACHE_ADDR");
    //     let generic_event_cache_port =
    //         std::env::var("EVENT_CACHE_PORT").expect("GENERIC_EVENT_CACHE_PORT");
    //
    //     format!("{}:{}", generic_event_cache_addr, generic_event_cache_port, )
    // };
    //
    // let cache = RedisCache::new(cache_address.to_owned())
    //     .await
    //     .expect("Could not create redis client");

    let fake_generator = vec![
        SysmonSubgraphGenerator::new(NopCache {}, SysmonSubgraphGeneratorMetrics::new(&env.service_name));
        10
    ];
    let mut fake_generator: [_; 10] = fake_generator.try_into().unwrap_or_else(|_|panic!("ahhh"));
    let mut fake_generator = &mut fake_generator;

    let serializer = vec![SubgraphSerializer::default(); 10];
    let mut serializer: [_; 10] = serializer.try_into().unwrap_or_else(|_| panic!("ahhh"));
    let mut serializer = &mut serializer;

    let mut s3_emitter = Vec::with_capacity(10);
    for _ in 0..10 {
        let emitter = S3EventEmitter::new(
            s3_client.clone(),
            destination_bucket.clone(),
            time_based_key_fn,
            move |_, _| async move { Ok(()) },
        );
        s3_emitter.push(emitter);
    }
    let mut s3_emitter: [_; 10] = s3_emitter.try_into().unwrap_or_else(|_| panic!("ahhh"));
    let s3_emitter = &mut s3_emitter;

    let s3_payload_retriever = vec![S3PayloadRetriever::new(
        |region_str| S3Client::new(Region::from_str(&region_str).expect("region_str")),
        ZstdDecoder::default(),
        MetricReporter::<Stdout>::new("sysmon-subgraph-generator"),
    ); 10];

    let mut s3_payload_retriever: [_; 10] = s3_payload_retriever.try_into().unwrap_or_else(|_|panic!("ahhh"));
    let s3_payload_retriever = &mut s3_payload_retriever;

    let cache = &mut [NopCache {}; 10];

    info!("Starting process_loop");
    sqs_executor::process_loop(
        std::env::var("QUEUE_URL").expect("QUEUE_URL"),
        cache,
        sqs_client.clone(),
        fake_generator,
        s3_payload_retriever,
        s3_emitter,
        serializer,
    ).await;

    info!("Exiting");
    println!("Exiting");
    // let metrics = SysmonSubgraphGeneratorMetrics::new(&env.service_name);
    //
    // if grapl_config::is_local() {
    //     info!("Starting sysmon-subgraph-generator locally");
    //     let generator = SysmonSubgraphGenerator::new(NopCache {}, metrics);
    //
    //     // run_graph_generator(
    //     //     generator,
    //     //     ZstdDecoder::default(),
    //     //     ConsumePolicyBuilder::default(),
    //     //     CompletionPolicy::new(
    //     //         1,                      // Buffer up to 1 message
    //     //         Duration::from_secs(1), // Buffer for up to 1 second
    //     //     ),
    //     //     MetricReporter::<Stdout>::new("sysmon-subgraph-generator"),
    //     // )
    //     // .await;
    // } else {
    //     info!("Starting sysmon-subgraph-generator in aws");
    //
    //     let generator = SysmonSubgraphGenerator::new(event_cache().await, metrics);
    //
    //     // let completion_policy = ConsumePolicyBuilder::default()
    //     //     .with_max_empty_receives(1)
    //     //     .with_stop_at(Duration::from_secs(10));
    //     //
    //     // run_graph_generator(
    //     //     generator,
    //     //     ZstdDecoder::default(),
    //     //     completion_policy,
    //     //     CompletionPolicy::new(10, Duration::from_secs(2)),
    //     //     MetricReporter::<Stdout>::new("sysmon-subgraph-generator"),
    //     // )
    //     // .await;
    // }

    Ok(())
}
