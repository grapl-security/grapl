#![allow(unused)]
#![allow(dead_code)]

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
use grapl_config::{env_helpers::{s3_event_emitters_from_env,
                                 FromEnv},
                   event_caches};
use grapl_graph_descriptions::{graph_description::{Edge,
                                                   EdgeList,
                                                   GeneratedSubgraphs,
                                                   Graph,
                                                   Node},
                               node::NodeT};
use grapl_observe::{dgraph_reporter::DgraphMetricReporter,
                    metric_reporter::{tag,
                                      MetricReporter}};
use grapl_service::{decoder::ZstdProtoDecoder,
                    serialization::SubgraphSerializer};
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

lazy_static! {
    /// timeout for dynamodb queries
    static ref DYNAMODB_QUERY_TIMEOUT: Duration = Duration::from_secs(3);
}

#[derive(Clone)]
struct GraphMerger<CacheT>
where
    CacheT: Cache + Clone + Send + Sync + 'static,
{
    mg_client: Arc<DgraphClient>,
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
        metric_reporter: MetricReporter<Stdout>,
        cache: CacheT,
    ) -> Self {
        let mg_client = DgraphClient::new(mg_alphas).expect("Failed to create dgraph client.");

        Self {
            mg_client: Arc::new(mg_client),
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
    let _s3_client = S3Client::from_env();

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
        GraphMerger::new(
            mg_alphas,
            MetricReporter::new(&env.service_name),
            cache[0].clone(),
        )
    })
    .await;

    let serializer = &mut make_ten(async { SubgraphSerializer::default() }).await;

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
    type InputEvent = GeneratedSubgraphs;
    type OutputEvent = Graph;
    type Error = GraphMergerError;

    async fn handle_event(
        &mut self,
        generated_subgraphs: GeneratedSubgraphs,
        _completed: &mut CompletedEvents,
    ) -> Result<Self::OutputEvent, Result<(Self::OutputEvent, Self::Error), Self::Error>> {
        let mut subgraph = Graph::new(0);

        for generated_subgraph in generated_subgraphs.subgraphs {
            subgraph.merge(&generated_subgraph);
        }

        if subgraph.is_empty() {
            warn!("Attempted to merge empty subgraph. Short circuiting.");
            return Ok(Graph::default());
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

        let uncached_nodes: Vec<Node> = {
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
                    Ok(results) => results,
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
                      if any of the cache responses are misses, the `.all(...)` should return false.
                      we negate this condition since we only want to *keep* nodes that have cache misses (as they need to be written)
                    */
                    !cache_results
                        .drain(0..std::cmp::min(num_of_predicates, cache_results.len()))
                        .all(|cache_result| match cache_result {
                            CacheResponse::Hit => true,
                            CacheResponse::Miss => false,
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
                         from,
                         to,
                         edge_name,
                     }| format!("{}:{}:{}", from, edge_name, to),
                )
                .collect();

            let cache_results: Vec<CacheResponse> =
                match self.cache.get_all(cacheable_edges.clone()).await {
                    Ok(result) => result,
                    Err(e) => {
                        error!(
                            "Error occurred when checking for cached edges in redis. {}",
                            e
                        );
                        // return an expected number of cache-misses
                        (0..all_edges.len()).map(|_| CacheResponse::Miss).collect()
                    }
                };

            // mark everything as cached (since we'll be processing it
            self.cache
                .store_all(
                    cacheable_edges
                        .into_iter()
                        .map(|item| item.as_bytes().to_vec())
                        .collect(),
                )
                .await;

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

        let r_edges: Vec<Edge> = {
            let dynamodb = DynamoDbClient::from_env();
            let r_edge_cache = &mut self.r_edge_cache;

            let f_edge_to_r_edge_target: HashMap<String, Option<String>> = uncached_edges
                .iter()
                .map(
                    |Edge {
                         from,
                         to,
                         edge_name,
                     }| {
                        (
                            edge_name.to_string(),
                            r_edge_cache.get(edge_name).map(String::from),
                        )
                    },
                )
                .collect();

            let unknown_r_edges: Vec<_> = f_edge_to_r_edge_target
                .iter()
                .filter(|(f_edge, r_edge)| r_edge.is_none())
                .map(|(f_edge, _)| f_edge.to_string())
                .collect();

            if !unknown_r_edges.is_empty() {
                let found_r_edges: HashMap<String, String> =
                    get_r_edges_from_dynamodb(&dynamodb, unknown_r_edges.clone())
                        .await
                        .map_err(|err| Err(err))?
                        .into_iter()
                        .filter_map(|(key, answer)| {
                            if answer.is_none() {
                                error!("Failed to fetch r_edge for f_edge {}", key);
                            }

                            answer.map(|value| (key, value))
                        })
                        .collect();

                r_edge_cache.extend(found_r_edges);
            }

            // fetch the r_edge_name and convert it into an Edge that points backwards along this edge_name
            uncached_edges
                .iter()
                .filter_map(
                    |Edge {
                         from,
                         to,
                         edge_name,
                     }| {
                        r_edge_cache
                            .get(edge_name)
                            .map(String::from)
                            .map(|r_edge_name| Edge {
                                from: to.clone(),
                                to: from.clone(),
                                edge_name: r_edge_name,
                            })
                    },
                )
                .collect()
        };

        let mut uncached_subgraph = Graph::new(0);

        for node in uncached_nodes {
            uncached_subgraph.add_node_without_edges(node);
        }

        for Edge {
            from,
            to,
            edge_name,
        } in uncached_edges.into_iter().chain(r_edges)
        {
            uncached_subgraph.add_edge(edge_name, from, to);
        }

        uncached_subgraph
            .perform_upsert(self.mg_client.clone())
            .await;

        Ok(uncached_subgraph)
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct EdgeMapping {
    r_edge: String,
}

/// Returns a HashMap of f_edge -> Optional r_edge entries from dynamodb
async fn get_r_edges_from_dynamodb(
    client: &DynamoDbClient,
    f_edges: Vec<String>,
) -> Result<HashMap<String, Option<String>>, GraphMergerError> {
    let schema_table_name = std::env::var("GRAPL_SCHEMA_TABLE").expect("GRAPL_SCHEMA_TABLE");

    let keys: Vec<HashMap<String, AttributeValue>> = f_edges
        .iter()
        .map(|f_edge| {
            let mut key_map = HashMap::new();
            key_map.insert(
                "f_edge".to_string(),
                AttributeValue {
                    s: Some(f_edge.to_string()),
                    ..Default::default()
                },
            );

            key_map
        })
        .collect();

    let keys_and_attributes = KeysAndAttributes {
        consistent_read: Some(true),
        keys,
        ..Default::default()
    };

    let mut request_items = HashMap::new();
    request_items.insert(schema_table_name.clone(), keys_and_attributes);

    let query = BatchGetItemInput {
        request_items,
        return_consumed_capacity: None,
    };

    /*
       1. Map timeout error
       2. Map Rusoto error
       3. Grab responses (HashMap<String, Vec<HashMap<String, AttributeValue>>>) (or return error)
       4. Pull the entire set of responses out for the schema table (Vec<HashMap<String, AttributeValue>>) (or return error)
    */
    let schema_table_response: Vec<HashMap<String, AttributeValue>> = client
        .batch_get_item_reliably(query)
        .timeout(*DYNAMODB_QUERY_TIMEOUT)
        .await
        .map_err(|e| GraphMergerError::Unexpected(e.to_string()))?
        .map_err(|e| GraphMergerError::Unexpected(e.to_string()))?
        .responses
        .ok_or(GraphMergerError::Unexpected(format!(
            "Failed to fetch results from dynamodb"
        )))?
        .remove(&schema_table_name)
        .ok_or(GraphMergerError::Unexpected(format!(
            "Missing data from expected table in dynamodb"
        )))?;

    /*
       1. Remove entries without f_edge and r_edge
       2. Grab both f_edge and r_edge properties as strings (`.s`)
       3. If both are present, return; otherwise error and filter entry out
    */
    Ok(schema_table_response
        .into_iter()
        .filter(|hashmap| hashmap.contains_key("f_edge") && hashmap.contains_key("r_edge"))
        .filter_map(|hashmap| {
            match (
                hashmap.get("f_edge").map(|item| item.s.clone()),
                hashmap.get("r_edge").map(|item| item.s.clone()),
            ) {
                (Some(Some(f_edge)), Some(r_edge)) => Some((f_edge, r_edge)),
                (Some(Some(f_edge)), _) => {
                    error!("Missing r_edge for f_edge ({}) in dynamodb schema.", f_edge);
                    None
                }
                (None, Some(Some(r_edge))) => {
                    error!(
                        "Failed to associate retrieved r_edge ({}) with an f_edge",
                        r_edge
                    );
                    None
                }
                _ => None,
            }
        })
        .collect())
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
