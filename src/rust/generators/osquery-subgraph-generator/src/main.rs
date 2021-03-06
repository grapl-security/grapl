#![type_length_limit = "1232619"]
mod generator;
mod metrics;
mod parsers;
mod serialization;
mod tests;

use graph_generator_lib::*;
use grapl_config::{env_helpers::{s3_event_emitters_from_env,
                                 FromEnv},
                   *};
use grapl_observe::metric_reporter::MetricReporter;
use grapl_service::serialization::GraphDescriptionSerializer;
use log::*;
use rusoto_sqs::SqsClient;
use sqs_executor::{event_retriever::S3PayloadRetriever,
                   make_ten,
                   s3_event_emitter::S3ToSqsEventNotifier,
                   time_based_key_fn};

use crate::{generator::OSQuerySubgraphGenerator,
            metrics::OSQuerySubgraphGeneratorMetrics,
            serialization::OSQueryLogDecoder};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (env, _guard) = grapl_config::init_grapl_env!();

    info!("Starting generic-subgraph-generator");

    let sqs_client = SqsClient::from_env();

    let cache = &mut event_caches(&env).await;

    let metrics = OSQuerySubgraphGeneratorMetrics::new(&env.service_name);
    let osquery_subgraph_generator =
        &mut make_ten(async { OSQuerySubgraphGenerator::new(cache[0].clone(), metrics.clone()) })
            .await;

    let serializer = &mut make_ten(async { GraphDescriptionSerializer::default() }).await;
    let s3_emitter =
        &mut s3_event_emitters_from_env(&env, time_based_key_fn, S3ToSqsEventNotifier::from(&env))
            .await;

    let s3_payload_retriever = &mut make_ten(async {
        S3PayloadRetriever::new(
            |region_str| grapl_config::env_helpers::init_s3_client(&region_str),
            OSQueryLogDecoder::default(),
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
