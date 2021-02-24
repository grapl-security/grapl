#![allow(unused)]
#![allow(dead_code)]
pub mod upserter;
pub mod upsert_util;
pub mod reverse_resolver;

use std::{collections::HashMap,
          fmt::Debug,
          io::Stdout,
          sync::{Arc,
                 Mutex},
          time::{Duration,
                 SystemTime,
                 UNIX_EPOCH}};
use crate::reverse_resolver::{get_r_edges_from_dynamodb, ReverseEdgeResolver};

use async_trait::async_trait;
use dgraph_tonic::{Client as DgraphClient,
                   Mutate,
                   Query};
use failure::{bail,
              Error};
use grapl_config::{env_helpers::{s3_event_emitters_from_env,
                                 FromEnv},
                   event_caches};
use grapl_graph_descriptions::graph_description::{Edge,
                                                  IdentifiedGraph,
                                                  IdentifiedNode,
                                                  MergedGraph,
                                                  MergedNode,
                                                  EdgeList};
use grapl_observe::{dgraph_reporter::DgraphMetricReporter,
                    metric_reporter::{tag,
                                      MetricReporter}};
use grapl_service::{decoder::ZstdProtoDecoder,
                    serialization::MergedGraphSerializer};

use grapl_utils::{future_ext::GraplFutureExt,
                  rusoto_ext::dynamodb::GraplDynamoDbClientExt};
use lazy_static::lazy_static;
use log::{error,
          info,
          warn};
use rusoto_dynamodb::{AttributeValue,
                      BatchGetItemInput,
                      DynamoDb,
                      DynamoDbClient,
                      GetItemInput,
                      KeysAndAttributes};
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

#[derive(Clone)]
struct GraphMerger<CacheT>
    where
        CacheT: Cache + Clone + Send + Sync + 'static,
{
    mg_client: Arc<DgraphClient>,
    reverse_edge_resolver: ReverseEdgeResolver,
    metric_reporter: MetricReporter<Stdout>,
    r_edge_cache: HashMap<String, String>,
    cache: CacheT,
}

impl<CacheT> GraphMerger<CacheT>
    where
        CacheT: Cache + Clone + Send + Sync + 'static,
{
    pub fn new(
        mg_alphas: Vec<String>,
        reverse_edge_resolver: ReverseEdgeResolver,
        metric_reporter: MetricReporter<Stdout>,
        cache: CacheT,
    ) -> Self {
        let mg_client = DgraphClient::new(mg_alphas).expect("Failed to create dgraph client.");

        Self {
            mg_client: Arc::new(mg_client),
            reverse_edge_resolver,
            metric_reporter,
            r_edge_cache: HashMap::with_capacity(256),
            cache,
        }
    }
}

fn time_based_key_fn(_event: &[u8]) -> String {
    info!("event length {}", _event.len());
    let cur_ms = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(n) => n.as_millis(),
        Err(_) => panic!("SystemTime before UNIX EPOCH!"),
    };

    let cur_day = cur_ms - (cur_ms % 86400);

    format!("{}/{}-{}", cur_day, cur_ms, uuid::Uuid::new_v4())
}

#[tracing::instrument]
async fn handler() -> Result<(), Box<dyn std::error::Error>> {
    let (env, _guard) = grapl_config::init_grapl_env!();
    info!("Starting graph-merger");

    let sqs_client = SqsClient::from_env();

    let cache = &mut event_caches(&env).await;

    // todo: the intitializer should give a cache to each service
    let graph_merger = &mut make_ten(async {
        let mg_alphas = grapl_config::mg_alphas();
        // Shoehorn `http://` in, if the user understandably forgot to do so
        let mg_alphas = mg_alphas
            .into_iter()
            .map(|mg_alpha| {
                if mg_alpha.contains("http://") {
                    mg_alpha
                } else {
                    format!("http://{}", mg_alpha)
                }
            })
            .collect();
        tracing::debug!(
            mg_alphas=?&mg_alphas,
            "Connecting to mg_alphas"
        );
        let dynamo = DynamoDbClient::from_env();
        let reverse_edge_resolver = ReverseEdgeResolver::new(
            dynamo, MetricReporter::new(&env.service_name), 1000,
        );
        GraphMerger::new(
            mg_alphas,
            reverse_edge_resolver,
            MetricReporter::new(&env.service_name),
            cache[0].clone(),
        )
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

#[derive(thiserror::Error, Debug)]
pub enum GraphMergerError {
    #[error("UnexpectedError")]
    Unexpected(String),
}

impl CheckedError for GraphMergerError {
    fn error_type(&self) -> Recoverable {
        Recoverable::Transient
    }
}

#[async_trait]
impl<CacheT> EventHandler for GraphMerger<CacheT>
    where
        CacheT: Cache + Clone + Send + Sync + 'static,
{
    type InputEvent = IdentifiedGraph;
    type OutputEvent = MergedGraph;
    type Error = GraphMergerError;

    async fn handle_event(
        &mut self,
        subgraph: Self::InputEvent,
        _completed: &mut CompletedEvents,
    ) -> Result<Self::OutputEvent, Result<(Self::OutputEvent, Self::Error), Self::Error>> {
        if subgraph.is_empty() {
            warn!("Attempted to merge empty subgraph. Short circuiting.");
            return Ok(MergedGraph::default());
        }

        info!(
            "handling new subgraph with {} nodes {} edges",
            subgraph.nodes.len(),
            subgraph.edges.len(),
        );

        /*
        1. cache check for nodes - remove hits
            1. for each node
                1. grab each predicate with a value that isn't null
                2. check if any predicate names DON'T hit cache
                3. if so, continue with node upsert
                4. else, remove node
        2. cache check for edges - remove hits
            1. for each 'edge'
                1. filter if `from:edge_name:to` is stored
                2. store key otherwise
            2.
        3. fetch r_edges and insert them into the subgraph
        4. perform upsert
        5. gang gang
         */

        let uncached_nodes: Vec<_> = {
            // list of nodes with different number of predicates
            // [[1, 2, 3], [4, 5], [6, 7, 8, 9]]
            // flatten
            // [1, 2, 3, 4, 5, 6, 7, 8, 9]
            // fetch from cache (node_key:name:value)
            // [h, m, m, h, h, m, m, h, h]
            // grab next n cache responses based on next n node predicates
            // [[h, m, m], [h, h], [m, m, h, h]]
            // zip
            // [([h, m, m], [1, 2, 3]), ([h, h], [4, 5]), ([m, m, h, h], [6, 7, 8, 9])]
            // ... filter on accumulative hit/miss -> contains a miss, return true (keep)
            // [([h, m, m], [1, 2, 3]), ([m, m, h, h], [6, 7, 8, 9])]
            // map nodes
            // [[1, 2, 3], [6, 7, 8, 9]]

            let predicate_cache_identities: Vec<_> = subgraph
                .nodes
                .iter()
                .flat_map(|(_, node)| node.get_cache_identities_for_predicates())
                .collect();

            let mut cache_results: Vec<CacheResponse> =
                match self.cache.get_all(predicate_cache_identities.clone()).await {
                    Ok(results) => results.into_iter().map(|(_, response)| response).collect(),
                    Err(e) => {
                        error!(
                            "Error occurred when checking for cached predicates in redis. {}",
                            e
                        );
                        (0..predicate_cache_identities.len())
                            .map(|_| CacheResponse::Miss)
                            .collect()
                    }
                };

            subgraph
                .nodes
                .into_iter()
                .map(|(_, node)| node)
                .filter(|node| {
                    let num_of_predicates = node.get_cache_identities_for_predicates().len();

                    /*
                     Retain nodes where there are any cache misses
                    */
                    cache_results
                        .iter()
                        .any(|cache_result| match cache_result {
                            CacheResponse::Hit => false,
                            CacheResponse::Miss => true,
                        })
                })
                .collect()
        };

        let uncached_edges: Vec<Edge> = {
            let all_edges: Vec<_> = subgraph
                .edges
                .into_iter()
                .flat_map(|(_, EdgeList { edges })| edges)
                .collect();

            let cacheable_edges: Vec<_> = all_edges
                .iter()
                .map(
                    |Edge {
                         from_node_key,
                         to_node_key,
                         edge_name,
                     }| format!("{}:{}:{}", from_node_key, edge_name, to_node_key),
                )
                .collect();

            let cache_results: Vec<CacheResponse> =
                match self.cache.get_all(cacheable_edges.clone()).await {
                    Ok(result) => result.into_iter().map(|(_, response)| response).collect(),
                    Err(e) => {
                        error!(
                            "Error occurred when checking for cached edges in redis. {}",
                            e
                        );
                        // return an expected number of cache-misses
                        (0..all_edges.len()).map(|_| CacheResponse::Miss).collect()
                    }
                };

            all_edges
                .into_iter()
                .zip(cache_results)
                .filter(|(_, cache_result)| match cache_result {
                    CacheResponse::Miss => true,
                    CacheResponse::Hit => false,
                })
                .map(|(edge, _)| edge)
                .collect()
        };

        let uncached_edges = self.reverse_edge_resolver.resolve_reverse_edges(uncached_edges).await
            .map_err(Err)?;

        let mut merged_graph = MergedGraph::new();
        let mut uncached_subgraph = IdentifiedGraph::new();

        for node in uncached_nodes {
            uncached_subgraph.add_node(node);
        }

        upserter::GraphMergeHelper{}
            .upsert_into(self.mg_client.clone(), &uncached_subgraph, &mut merged_graph)
            .await;

        Ok(merged_graph)
    }
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    handler().await?;
    Ok(())
}

trait AnyhowFailure {
    fn into_failure(self) -> Error;
}

impl AnyhowFailure for anyhow::Error {
    fn into_failure(self) -> Error {
        failure::Error::from_boxed_compat(From::from(self))
    }
}
