#![type_length_limit = "1232619"]

mod generator;
mod models;
mod serialization;
mod tests;

use sqs_executor::cache::{Cache, NopCache};

use tracing::*;

use grapl_config::{event_cache, event_caches};
use sqs_executor::event_decoder::PayloadDecoder;

use crate::generator::GenericSubgraphGenerator;
use crate::serialization::ZstdJsonDecoder;
use grapl_observe::metric_reporter::MetricReporter;
use std::io::Stdout;
use std::time::Duration;
use grapl_config::env_helpers::s3_event_emitters_from_env;
use sqs_executor::event_retriever::S3PayloadRetriever;
use rusoto_s3::S3Client;
use rusoto_core::Region;
use rusoto_sqs::SqsClient;
use grapl_config::env_helpers::FromEnv;
use std::str::FromStr;
use sqs_executor::{make_ten, time_based_key_fn};
use grapl_service::serialization::zstd_proto::SubgraphSerializer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let env = grapl_config::init_grapl_env!();

    info!("Starting generic-subgraph-generator");

    let sqs_client = SqsClient::from_env();
    let s3_client = S3Client::from_env();

    let destination_bucket = grapl_config::dest_bucket();
    let cache = &mut event_caches(&env).await;

    let generic_subgraph_generator = &mut make_ten(async {
        GenericSubgraphGenerator::new(NopCache {})
    })
        .await;

    let serializer = &mut make_ten(async { SubgraphSerializer::default() }).await;
    let s3_emitter = &mut s3_event_emitters_from_env(&env, time_based_key_fn).await;

    let s3_payload_retriever = &mut make_ten(async {
        S3PayloadRetriever::new(
            |region_str| S3Client::new(Region::from_str(&region_str).expect("region_str")),
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
        generic_subgraph_generator,
        s3_payload_retriever,
        s3_emitter,
        serializer,
        MetricReporter::new(&env.service_name),
    )
        .await;

    info!("Exiting");
    println!("Exiting");


    Ok(())
}
