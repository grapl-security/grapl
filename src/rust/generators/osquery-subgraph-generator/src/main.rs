#![type_length_limit = "1232619"]

mod generator;
mod metrics;
mod parsers;

mod tests;

use crate::generator::OSQuerySubgraphGenerator;
use crate::metrics::OSQuerySubgraphGeneratorMetrics;

use graph_generator_lib::*;
use grapl_config::env_helpers::{s3_event_emitters_from_env, FromEnv};
use grapl_config::*;
use grapl_observe::metric_reporter::MetricReporter;
use grapl_service::decoder::ZstdJsonDecoder;
use grapl_service::serialization::zstd_proto_graph::SubgraphSerializer;
use log::*;
use rusoto_core::Region;
use rusoto_s3::S3Client;
use rusoto_sqs::SqsClient;
use sqs_executor::cache::NopCache;
use sqs_executor::event_retriever::S3PayloadRetriever;
use sqs_executor::s3_event_emitter::S3ToSqsEventNotifier;
use sqs_executor::{make_ten, time_based_key_fn};
use std::io::Stdout;
use std::str::FromStr;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let env = grapl_config::init_grapl_env!();

    info!("Starting generic-subgraph-generator");

    let sqs_client = SqsClient::from_env();

    let destination_bucket = grapl_config::dest_bucket();
    let cache = &mut event_caches(&env).await;

    let metrics = OSQuerySubgraphGeneratorMetrics::new(&env.service_name);
    let osquery_subgraph_generator =
        &mut make_ten(async { OSQuerySubgraphGenerator::new(cache[0].clone(), metrics.clone()) })
            .await;

    let serializer = &mut make_ten(async { SubgraphSerializer::default() }).await;
    let s3_emitter =
        &mut s3_event_emitters_from_env(&env, time_based_key_fn, S3ToSqsEventNotifier::from_env())
            .await;

    let s3_payload_retriever = &mut make_ten(async {
        S3PayloadRetriever::new(
            |region_str| grapl_config::env_helpers::init_s3_client(&region_str),
            ZstdJsonDecoder::default(),
            MetricReporter::new(&env.service_name),
        )
    })
    .await;

    info!("Starting process_loop");
    sqs_executor::process_loop(
        grapl_config::source_queue_url(),
        grapl_config::dead_letter_queue_url(),
        cache,
        sqs_client.clone(),
        osquery_subgraph_generator,
        s3_payload_retriever,
        s3_emitter,
        serializer,
        MetricReporter::new(&env.service_name),
    )
    .await;

    info!("Exiting");

    Ok(())
}
