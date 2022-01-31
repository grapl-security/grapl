use grapl_config::{
    env_helpers::{
        s3_event_emitters_from_env,
        FromEnv,
    },
    event_caches,
};
use grapl_observe::metric_reporter::MetricReporter;
use grapl_service::{
    decoder::ProtoDecoder,
    serialization::MergedGraphSerializer,
};
use rusoto_dynamodb::DynamoDbClient;
use rusoto_sqs::SqsClient;
use sqs_executor::{
    make_ten,
    s3_event_retriever::S3PayloadRetriever,
};
use tracing::info;

use crate::{
    reverse_resolver::ReverseEdgeResolver,
    service::{
        time_based_key_fn,
        GraphMerger,
    },
};

pub mod reverse_resolver;
pub mod service;
pub mod upsert_util;
pub mod upserter;

#[tracing::instrument]
async fn handler() -> Result<(), Box<dyn std::error::Error>> {
    let (env, _guard) = grapl_config::init_grapl_env!();
    info!("Starting graph-merger");

    let sqs_client = SqsClient::from_env();

    let cache = &mut event_caches(&env).await;

    let mg_alphas = grapl_config::mg_alphas();

    let graph_merger = &mut make_ten(async {
        let mg_alphas_copy = mg_alphas.clone();
        tracing::debug!(
            mg_alphas=?&mg_alphas_copy,
            "Connecting to mg_alphas"
        );
        let dynamo = DynamoDbClient::from_env();
        let reverse_edge_resolver =
            ReverseEdgeResolver::new(dynamo, MetricReporter::new(&env.service_name), 1000);
        GraphMerger::new(mg_alphas_copy, reverse_edge_resolver)
    })
    .await;

    let serializer = &mut make_ten(async { MergedGraphSerializer::default() }).await;

    let s3_emitter = &mut s3_event_emitters_from_env(&env, time_based_key_fn).await;

    let s3_payload_retriever = &mut make_ten(async {
        S3PayloadRetriever::new(
            |region_str| grapl_config::env_helpers::init_s3_client(&region_str),
            ProtoDecoder::default(),
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
        graph_merger,
        s3_payload_retriever,
        s3_emitter,
        serializer,
        MetricReporter::new(&env.service_name),
    )
    .await;

    info!("Exiting");

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    handler().await?;
    Ok(())
}
