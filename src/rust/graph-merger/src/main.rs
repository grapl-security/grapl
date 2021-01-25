#![allow(unused_must_use)]

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
use futures::future::FutureExt;
use grapl_config::{env_helpers::{s3_event_emitters_from_env,
                                 FromEnv},
                   event_caches};
use grapl_graph_descriptions::graph_description::{Edge,
                                                  IdentifiedGraph,
                                                  IdentifiedNode,
                                                  MergedGraph,
                                                  MergedNode};
use grapl_observe::{dgraph_reporter::DgraphMetricReporter,
                    metric_reporter::{tag,
                                      MetricReporter}};
use grapl_service::{decoder::ZstdProtoDecoder,
                    serialization::MergedGraphSerializer};
use log::{error,
          info,
          warn};
use lru_cache::LruCache;
use rusoto_dynamodb::{AttributeValue,
                      DynamoDb,
                      DynamoDbClient,
                      GetItemInput};
use rusoto_sqs::SqsClient;
use serde::{Deserialize,
            Serialize};
use serde_json::{json,
                 Value};
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

fn generate_edge_insert(from: &str, to: &str, edge_name: &str) -> dgraph_tonic::Mutation {
    let mu = json!({
        "uid": from,
        edge_name: {
            "uid": to
        }
    });

    let mut mutation = dgraph_tonic::Mutation::new();
    mutation.commit_now = true;
    mutation.set_set_json(&mu);

    mutation
}

async fn node_key_to_uid(
    dg: &DgraphClient,
    metric_reporter: &mut MetricReporter<Stdout>,
    uid_cache: &mut UidCache,
    node_key: &str,
) -> Result<Option<String>, Error> {
    if uid_cache.is_empty() {
        tracing::debug!("uid_cache is empty");
    }
    if let Some(uid) = uid_cache.get(node_key) {
        let _ =
            metric_reporter.counter("node_key_to_uid.cache.count", 1.0, 0.1, &[tag("hit", true)]);
        return Ok(Some(uid));
    } else {
        let _ = metric_reporter.counter(
            "node_key_to_uid.cache.count",
            1.0,
            0.1,
            &[tag("hit", false)],
        );
    }

    let mut txn = dg.new_read_only_txn();

    const QUERY: &str = r"
       query q0($a: string)
    {
        q0(func: eq(node_key, $a), first: 1) @cascade {
            uid,
            dgraph.type,
        }
    }";

    let mut vars = HashMap::new();
    vars.insert("$a".to_string(), node_key.to_string());

    let query_res = tokio::time::timeout(Duration::from_secs(3), txn.query_with_vars(QUERY, vars))
        .await?
        .map_err(AnyhowFailure::into_failure)?;

    // todo: is there a way to differentiate this query metric from others?
    metric_reporter
        .query(&query_res, &[])
        .unwrap_or_else(|e| error!("query metric failed: {}", e));

    let query_res: Value = serde_json::from_slice(&query_res.json)?;

    let uid = query_res
        .get("q0")
        .and_then(|res| res.get(0))
        .and_then(|uid| uid.get("uid"))
        .and_then(|uid| uid.as_str())
        .map(String::from);
    if let Some(uid) = uid {
        uid_cache.store(node_key.to_string(), uid.to_owned());
        Ok(Some(uid))
    } else {
        Ok(None)
    }
}

async fn upsert_node<CacheT>(
    dg: &DgraphClient,
    cache: &mut CacheT,
    uid_cache: &mut UidCache,
    metric_reporter: &mut MetricReporter<Stdout>,
    node: IdentifiedNode,
) -> Result<String, Error>
where
    CacheT: Cache,
{
    let query = format!(
        r#"
                {{
                  p as var(func: eq(node_key, "{}"), first: 1)
                }}
                "#,
        &node.node_key
    );

    let node_key = node.clone_node_key();
    let mut set_json: serde_json::Value = node.into_json();
    let mut node_types = vec![set_json["dgraph.type"].as_str().unwrap().clone()];
    node_types.extend_from_slice(&["Entity", "Base"]);
    set_json["dgraph.type"] = node_types.into();

    set_json["uid"] = "uid(p)".into();
    let cache_key = serde_json::to_string(&set_json).expect("mutation was invalid json");

    match cache.get(cache_key.clone()).await {
        Ok(CacheResponse::Miss) => {
            let _ =
                metric_reporter.counter("upsert_node.cache.count", 1.0, 0.5, &[tag("hit", false)]);
            set_json["last_index_time"] = (SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Something is very wrong with the system clock")
                .as_millis() as u64)
                .into();

            let mut mu = dgraph_tonic::Mutation::new();
            mu.commit_now = true;
            mu.set_set_json(&set_json);
            tracing::debug!(
                node_key=?node_key,
                "Performing upsert"
            );
            let mut txn = dg.new_mutated_txn();
            let upsert_res =
                match tokio::time::timeout(Duration::from_secs(10), txn.upsert(query, mu)).await? {
                    Ok(res) => res,
                    Err(e) => {
                        tokio::time::timeout(Duration::from_secs(10), txn.discard())
                            .await?
                            .map_err(AnyhowFailure::into_failure)?;
                        return Err(e.into_failure().into());
                    }
                };

            metric_reporter
                .mutation(&upsert_res, &[])
                .unwrap_or_else(|e| error!("mutation metric failed: {}", e));
            tokio::time::timeout(Duration::from_secs(3), txn.commit())
                .await?
                .map_err(AnyhowFailure::into_failure)?;

            info!(
                "Upsert res for {}, set_json: {} upsert_res: {:?}",
                node_key,
                set_json.to_string(),
                upsert_res,
            );
            cache.store(cache_key.into_bytes());
        }
        Err(e) => error!("Failed to get upsert from cache: {}", e),
        Ok(CacheResponse::Hit) => {
            let _ =
                metric_reporter.counter("upsert_node.cache.count", 1.0, 0.1, &[tag("hit", true)]);
        }
    }

    match node_key_to_uid(dg, metric_reporter, uid_cache, &node_key).await? {
        Some(uid) => Ok(uid),
        None => bail!("Could not retrieve uid after upsert for {}", &node_key),
    }
}

#[derive(Debug, Clone)]
struct UidCache {
    cache: Arc<Mutex<LruCache<String, String>>>,
}

impl Default for UidCache {
    fn default() -> Self {
        Self {
            cache: Arc::new(Mutex::new(LruCache::new(100_000))),
        }
    }
}

impl UidCache {
    fn is_empty(&self) -> bool {
        let self_cache = self.cache.lock().unwrap();
        self_cache.is_empty()
    }
    fn get(&self, node_key: &str) -> Option<String> {
        let mut self_cache = self.cache.lock().unwrap();
        let cache_res = self_cache.get_mut(node_key);
        if cache_res.is_some() {
            tracing::debug!("Cache hit");
        } else {
            tracing::debug!("Cache miss")
        }
        cache_res.cloned()
    }
    fn store(&mut self, node_key: String, uid: String) {
        let mut self_cache = self.cache.lock().unwrap();
        self_cache.insert(node_key, uid);
    }
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
    uid_cache: UidCache,
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
        let mg_client = DgraphClient::new(mg_alphas).expect("Failed to create dgraphclient.");

        Self {
            mg_client: Arc::new(mg_client),
            metric_reporter,
            r_edge_cache: HashMap::with_capacity(100),
            uid_cache: UidCache::default(),
            cache,
        }
    }
}

async fn upsert_edge<CacheT>(
    mg_client: &DgraphClient,
    metric_reporter: &mut MetricReporter<Stdout>,
    cache: &mut CacheT,
    to: &str,
    from: &str,
    edge_name: &str,
) -> Result<(), failure::Error>
where
    CacheT: Cache,
{
    let cache_key = format!("{}{}{}", &to, &from, &edge_name);
    match cache.get(cache_key.as_bytes().to_owned()).await {
        Ok(CacheResponse::Hit) => return Ok(()),
        Ok(CacheResponse::Miss) => (),
        Err(e) => error!("Failed to retrieve from edge_cache: {:?}", e),
    };

    let mu = generate_edge_insert(&to, &from, &edge_name);
    let txn = mg_client.new_mutated_txn();
    let mut_res = tokio::time::timeout(Duration::from_secs(10), txn.mutate_and_commit_now(mu))
        .await?
        .map_err(AnyhowFailure::into_failure)?;
    metric_reporter
        .mutation(&mut_res, &[])
        .unwrap_or_else(|e| error!("edge mutation metric failed: {}", e));

    cache.store(cache_key.into_bytes()).await;
    Ok(())
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
        subgraph: IdentifiedGraph,
        _completed: &mut CompletedEvents,
    ) -> Result<Self::OutputEvent, Result<(Self::OutputEvent, Self::Error), Self::Error>> {
        if subgraph.is_empty() {
            warn!("Attempted to merge empty subgraph. Short circuiting.");
            return Ok(MergedGraph::default());
        }

        info!(
            "handling new subgraphwith {} nodes {} edges",
            subgraph.nodes.len(),
            subgraph.edges.len(),
        );

        let mut merged_graph = MergedGraph::default();

        let mut upsert_res = None;

        let node_key_to_uid_map = &mut self.uid_cache;
        let mut upserts = Vec::with_capacity(subgraph.nodes.len());
        for node in subgraph.nodes.values() {
            match self
                .cache
                .get(
                    subgraph.nodes[node.node_key.as_str()]
                        .clone()
                        .into_json()
                        .to_string(),
                )
                .await
            {
                Ok(CacheResponse::Hit) => {
                    info!("Got cache hit for old_node_key, skipping node.");
                    continue;
                }
                Err(e) => warn!("Failed to retrieve from cache: {:?}", e),
                _ => (),
            };
            upserts.push(
                upsert_node(
                    &self.mg_client,
                    &mut self.cache,
                    node_key_to_uid_map,
                    &mut self.metric_reporter,
                    node.clone(),
                )
                .map(move |u| (node.clone(), u))
                .await,
            )
        }

        let mut upsert_count = 0u32;
        let mut failed_count = 0u32;
        for (node, upsert) in upserts.into_iter() {
            let new_uid = match upsert {
                Ok(new_uid) => {
                    upsert_count += 1;
                    new_uid
                }
                Err(e) => {
                    failed_count += 1;
                    error!("upsert_error: {}", e);
                    upsert_res = Some(e);
                    continue;
                }
            };
            node_key_to_uid_map.store(node.clone_node_key(), new_uid.clone());

            let new_uid = new_uid.trim_start_matches("0x");
            let new_uid: u64 = u64::from_str_radix(new_uid, 16).expect("todo: raise parseinterror");
            let merged_node = MergedNode::from(node, new_uid);
            merged_graph.add_node(merged_node);
        }

        if (upsert_count == 0) && upsert_res.is_some() {
            return Err(Err(GraphMergerError::Unexpected(format!(
                "All nodes failed to upsert {:?}",
                upsert_res
            ))));
        }

        info!(
            "Upserted: {} nodes, {} failures",
            upsert_count, failed_count
        );

        info!("Inserting edges {}", subgraph.edges.len());
        let dynamodb = DynamoDbClient::from_env();

        let unmerged_edges: Vec<_> = subgraph
            .edges
            .values()
            .map(|e| &e.edges)
            .flatten()
            .map(|edge| edge.clone())
            .collect();

        let merged_edges = upsert_edges(
            &unmerged_edges[..],
            &self.mg_client,
            &mut self.metric_reporter,
            node_key_to_uid_map,
            &mut self.cache,
        )
        .await
        .map_err(|e| Err(e))?;

        let reversed_edges = reverse_edges(&unmerged_edges[..], &dynamodb, &mut self.r_edge_cache)
            .await
            .map_err(|e| Err(e))?;

        let merged_reverse_edges = upsert_edges(
            &reversed_edges[..],
            &self.mg_client,
            &mut self.metric_reporter,
            node_key_to_uid_map,
            &mut self.cache,
        )
        .await
        .map_err(|e| Err(e))?;

        for edge in merged_edges.into_iter() {
            merged_graph.add_merged_edge(edge);
        }
        for edge in merged_reverse_edges.into_iter() {
            merged_graph.add_merged_edge(edge);
        }

        Ok(merged_graph)
    }
}

async fn reverse_edge(
    from_node_key: String,
    to_node_key: String,
    edge_name: String,
    dynamodb: &DynamoDbClient,
    r_edge_cache: &mut HashMap<String, String>,
) -> Result<Edge, GraphMergerError> {
    let r_edge = match r_edge_cache.get(&edge_name.to_string()) {
        Some(r_edge) => r_edge.to_string(),
        None => match get_r_edge(&dynamodb, edge_name.clone()).await? {
            Some(r_edge) => r_edge.to_string(),
            None => {
                return Err(GraphMergerError::Unexpected(format!(
                    "No reverse edge for: {}",
                    &edge_name
                )))
            }
        },
    };

    if r_edge.is_empty() {
        return Err(GraphMergerError::Unexpected(format!(
            "Empty reverse edge for: {}",
            &edge_name
        )));
    }

    Ok(Edge {
        from: to_node_key,
        to: from_node_key,
        edge_name: r_edge.clone(),
    })
}

async fn reverse_edges(
    edges: &[Edge],
    dynamodb: &DynamoDbClient,
    cache: &mut HashMap<String, String>,
) -> Result<Vec<Edge>, GraphMergerError> {
    let mut reversed_edges = Vec::with_capacity(edges.len());
    for edge in edges {
        reversed_edges.push(
            reverse_edge(
                edge.from.clone(),
                edge.to.clone(),
                edge.edge_name.clone(),
                dynamodb,
                cache,
            )
            .await?,
        )
    }

    Ok(reversed_edges)
}

async fn get_edge_uids(
    from_node_key: &str,
    to_node_key: &str,
    mg_client: &DgraphClient,
    metric_reporter: &mut MetricReporter<Stdout>,
    cache: &mut UidCache,
) -> Result<(String, String), GraphMergerError> {
    let from_uid = match node_key_to_uid(&mg_client, metric_reporter, cache, from_node_key)
        .await
        .map_err(|e| GraphMergerError::Unexpected(e.to_string()))?
    {
        Some(from_uid) => from_uid,
        None => {
            return Err(GraphMergerError::Unexpected(format!(
                "Could not resolve edge: {:?}",
                from_node_key
            )));
        }
    };

    let to_uid = match node_key_to_uid(&mg_client, metric_reporter, cache, to_node_key)
        .await
        .map_err(|e| GraphMergerError::Unexpected(e.to_string()))?
    {
        Some(to_uid) => to_uid,
        None => {
            return Err(GraphMergerError::Unexpected(format!(
                "Could not resolve edge: {:?}",
                from_node_key
            )));
        }
    };
    Ok((from_uid, to_uid))
}

use grapl_graph_descriptions::graph_description::MergedEdge;

async fn upsert_edges<CacheT>(
    unmerged_edges: &[Edge],
    mg_client: &DgraphClient,
    metric_reporter: &mut MetricReporter<Stdout>,
    uid_cache: &mut UidCache,
    cache: &mut CacheT,
) -> Result<Vec<MergedEdge>, GraphMergerError>
where
    CacheT: Cache,
{
    let mut edge_mutations = Vec::with_capacity(unmerged_edges.len());
    for edge in unmerged_edges {
        let (from_uid, to_uid) =
            get_edge_uids(&edge.from, &edge.to, mg_client, metric_reporter, uid_cache).await?;

        upsert_edge(
            mg_client,
            metric_reporter,
            cache,
            &to_uid,
            &from_uid,
            &edge.edge_name,
        )
        .await
        .map_err(|e| GraphMergerError::Unexpected(e.to_string()))?;
        edge_mutations.push(MergedEdge {
            from_node_key: edge.from.clone(),
            from_uid,
            to_node_key: edge.to.clone(),
            to_uid,
            edge_name: edge.edge_name.clone(),
        });
    }

    Ok(edge_mutations)
}

#[derive(Debug, Serialize, Deserialize)]
struct EdgeMapping {
    r_edge: String,
}

async fn get_r_edge(
    client: &DynamoDbClient,
    f_edge: String,
) -> Result<Option<String>, GraphMergerError> {
    let mut key = HashMap::new();

    key.insert(
        "f_edge".to_owned(),
        AttributeValue {
            s: Some(f_edge.to_owned()),
            ..Default::default()
        },
    );

    let query = GetItemInput {
        consistent_read: Some(true),
        table_name: std::env::var("GRAPL_SCHEMA_TABLE").expect("GRAPL_SCHEMA_TABLE"),
        key,
        ..Default::default()
    };

    let item = tokio::time::timeout(Duration::from_secs(2), client.get_item(query))
        .await
        .map_err(|e| GraphMergerError::Unexpected(e.to_string()))?
        .map_err(|e| GraphMergerError::Unexpected(e.to_string()))?
        .item;
    match item {
        Some(item) => {
            let mapping: EdgeMapping = serde_dynamodb::from_hashmap(item.clone())
                .map_err(|e| GraphMergerError::Unexpected(e.to_string()))?;
            Ok(Some(mapping.r_edge))
        }
        None => {
            error!("Missing r_edge for: {}", f_edge);
            Ok(None)
        }
    }
}

#[derive(Clone, Default)]
struct HashCache {
    cache: Arc<Mutex<std::collections::HashSet<Vec<u8>>>>,
}

#[derive(thiserror::Error, Debug)]
pub enum HashCacheError {
    // standin until the never type (`!`) is stable
    #[error("HashCacheError.Unreachable")]
    Unreachable,
}

impl CheckedError for HashCacheError {
    fn error_type(&self) -> Recoverable {
        panic!("HashCacheError should be unreachable")
    }
}

#[async_trait]
impl sqs_executor::cache::Cache for HashCache {
    type CacheErrorT = HashCacheError;

    async fn get<CA: Cacheable + Send + Sync + 'static>(
        &mut self,
        cacheable: CA,
    ) -> Result<CacheResponse, Self::CacheErrorT> {
        let self_cache = self.cache.lock().unwrap();

        let id = cacheable.identity();
        if self_cache.contains(&id) {
            Ok(CacheResponse::Hit)
        } else {
            Ok(CacheResponse::Miss)
        }
    }
    async fn store(&mut self, identity: Vec<u8>) -> Result<(), Self::CacheErrorT> {
        let mut self_cache = self.cache.lock().unwrap();
        self_cache.insert(identity);
        Ok(())
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
