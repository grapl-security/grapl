#![allow(unused)]
#![allow(dead_code)]

pub mod service;

use std::{collections::HashMap,
          fmt::Debug,
          io::Stdout,
          sync::{Arc,
                 Mutex},
          time::{Duration,
                 SystemTime,
                 UNIX_EPOCH}};

use async_trait::async_trait;
use dgraph_tonic::{Client as DgraphClient,
                   Mutate,
                   Query};
use failure::{bail,
              Error};
use graph_merger_lib;
use grapl_config::{env_helpers::{s3_event_emitters_from_env,
                                 FromEnv},
                   event_caches};
use grapl_graph_descriptions::{graph_description::{IdentifiedEdge,
                                                   IdentifiedEdgeList,
                                                   IdentifiedGraph,
                                                   IdentifiedNode,
                                                   MergedGraph,
                                                   MergedNode},
                               graph_mutation_service::graph_mutation_rpc_client::GraphMutationRpcClient};
use grapl_observe::{dgraph_reporter::DgraphMetricReporter,
                    metric_reporter::{tag,
                                      MetricReporter}};
use grapl_service::{decoder::ZstdProtoDecoder,
                    serialization::MergedGraphSerializer};
use grapl_utils::{future_ext::GraplFutureExt,
                  rusoto_ext::dynamodb::GraplDynamoDbClientExt};
use lazy_static::lazy_static;
use rusoto_s3::S3Client;
use rusoto_sqs::SqsClient;
use serde::{Deserialize,
            Serialize};
use serde_json::Value;
use sqs_executor::{cache::{Cache,
                           CacheResponse,
                           Cacheable},
                   errors::{CheckedError,
                            Recoverable},
                   event_handler::{CompletedEvents,
                                   EventHandler},
                   event_retriever::S3PayloadRetriever,
                   make_ten,
                   s3_event_emitter::S3ToSqsEventNotifier};
use tonic::transport::Channel;
use tracing::{error,
              info,
              warn};

use crate::service::{time_based_key_fn,
                     GraphMerger};

#[tracing::instrument]
async fn handler() -> Result<(), Box<dyn std::error::Error>> {
    let (env, _guard) = grapl_config::init_grapl_env!();
    info!("Starting graph-merger");

    let sqs_client = SqsClient::from_env();

    let cache = &mut event_caches(&env).await;

    let mutation_endpoint = grapl_config::mutation_endpoint();
    // todo: the intitializer should give a cache to each service
    let graph_merger = &mut make_ten(async {
        tracing::debug!(
            mutation_endpoint=?&mutation_endpoint,
            "Connecting to mutation_endpoint"
        );
        let graph_mutation_client: GraphMutationRpcClient<Channel> =
            GraphMutationRpcClient::connect(mutation_endpoint)
                .await
                .expect("Failed to connect to graph-mutation-service");

        GraphMerger::new(
            graph_mutation_client,
            MetricReporter::new(&env.service_name),
            cache[0].clone(),
        )
        .await
    })
    .await;

    let serializer = &mut make_ten(async { MergedGraphSerializer::default() }).await;

    let s3_emitter =
        &mut s3_event_emitters_from_env(&env, time_based_key_fn, S3ToSqsEventNotifier::from(&env))
            .await;

    let s3_payload_retriever = &mut make_ten(async {
        S3PayloadRetriever::new(
            |region_str| grapl_config::env_helpers::init_s3_client(&region_str),
            ZstdProtoDecoder::default(),
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
