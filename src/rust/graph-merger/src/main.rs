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
use grapl_graph_descriptions::{graph_description::{GeneratedSubgraphs,
                                                   Graph,
                                                   Node},
                               node::NodeT};
use grapl_observe::{dgraph_reporter::DgraphMetricReporter,
                    metric_reporter::{tag,
                                      MetricReporter}};
use grapl_service::{decoder::ZstdProtoDecoder,
                    serialization::SubgraphSerializer};
use log::{error,
          info,
          warn};
use lru_cache::LruCache;
use rusoto_dynamodb::{AttributeValue,
                      DynamoDb,
                      DynamoDbClient,
                      GetItemInput};
use rusoto_s3::S3Client;
use rusoto_sqs::SqsClient;
use serde::{Deserialize,
            Serialize};
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
use serde_json::Value;
/*
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
}*/

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
    node: Node,
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
        node.get_node_key()
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
        let mg_client = DgraphClient::new(mg_alphas).expect("Failed to create dgraph client.");

        Self {
            mg_client: Arc::new(mg_client),
            metric_reporter,
            r_edge_cache: HashMap::with_capacity(100),
            uid_cache: UidCache::default(),
            cache,
        }
    }
}

/*
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
}*/

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

        //let mut identities = Vec::with_capacity(subgraph.nodes.len() + subgraph.edges.len());

        info!(
            "handling new subgraph with {} nodes {} edges",
            subgraph.nodes.len(),
            subgraph.edges.len(),
        );

        //let mut upsert_res = None;
        //let mut edge_res = None;

        //let mut node_key_to_uid_map = HashMap::new();
        //let mut upserts = Vec::with_capacity(subgraph.nodes.len());

        subgraph.perform_upsert(self.mg_client.clone()).await;
/*
        for node in subgraph.nodes.values() {
            match self
                .cache
                .get(
                    subgraph.nodes[node.get_node_key()]
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
                .map(move |u| (node.clone_node_key(), u))
                .await,
            )
        }*/
/*
        for (node_key, upsert) in upserts.into_iter() {
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

            node_key_to_uid_map.store(node_key, new_uid);
        }

        if (upsert_count == 0) && upsert_res.is_some() {
            return Err(Err(GraphMergerError::Unexpected(format!(
                "All nodes failed to upsert {:?}",
                upsert_res
            ))));
        }

        info!("Upserted: {} nodes", node_key_to_uid_map.len());
*/
        /*
        info!("Inserting edges {}", subgraph.edges.len());
        let dynamodb = DynamoDbClient::from_env();

        let mut edge_mutations: Vec<_> = vec![];

        let flattened_edges: Vec<_> = subgraph
            .edges
            .values()
            .map(|e| &e.edges)
            .flatten()
            .collect();
        for edge in flattened_edges.into_iter() {
            match (
                node_key_to_uid_map.get(&edge.from[..]),
                node_key_to_uid_map.get(&edge.to[..]),
            ) {
                (Some(from), Some(to)) if from == to => {
                    let err = format!(
                        "From and To can not be the same uid {} {} {} {} {}",
                        from,
                        to,
                        &edge.from[..],
                        &edge.to[..],
                        &edge.edge_name
                    );
                    error!("{}", err);
                    edge_res = Some(err);
                }
                (Some(from), Some(to)) => {
                    info!("Upserting edge: {} {} {}", &from, &to, &edge.edge_name);
                    edge_mutations.push((from.to_owned(), to.to_owned(), &edge.edge_name));
                }
                (Some(from), None) => {
                    match node_key_to_uid(
                        &self.mg_client,
                        &mut self.metric_reporter,
                        node_key_to_uid_map,
                        &edge.from[..],
                    )
                    .await
                    {
                        Ok(Some(to)) => {
                            edge_mutations.push((from.to_owned(), to.to_owned(), &edge.edge_name))
                        }
                        Ok(None) => edge_res = Some("Edge to uid failed".to_string()),
                        Err(e) => edge_res = Some(e.to_string()),
                    }
                }
                (None, Some(to)) => {
                    match node_key_to_uid(
                        &self.mg_client,
                        &mut self.metric_reporter,
                        node_key_to_uid_map,
                        &edge.to[..],
                    )
                    .await
                    {
                        Ok(Some(from)) => {
                            edge_mutations.push((from.to_owned(), to.to_owned(), &edge.edge_name))
                        }
                        Ok(None) => edge_res = Some("Edge to uid failed".to_string()),
                        Err(e) => edge_res = Some(e.to_string()),
                    }
                }
                (None, None) => {
                    let from = match node_key_to_uid(
                        &self.mg_client,
                        &mut self.metric_reporter,
                        node_key_to_uid_map,
                        &edge.from[..],
                    )
                    .await
                    {
                        Ok(Some(from)) => from,
                        Ok(None) => {
                            edge_res = Some("Edge to uid failed".to_string());
                            continue;
                        }
                        Err(e) => {
                            edge_res = Some(e.to_string());
                            continue;
                        }
                    };

                    let to = match node_key_to_uid(
                        &self.mg_client,
                        &mut self.metric_reporter,
                        node_key_to_uid_map,
                        &edge.to[..],
                    )
                    .await
                    {
                        Ok(Some(to)) => to,
                        Ok(None) => {
                            edge_res = Some("Edge to uid failed".to_string());
                            continue;
                        }
                        Err(e) => {
                            edge_res = Some(e.to_string());
                            continue;
                        }
                    };

                    edge_mutations.push((from.to_owned(), to.to_owned(), &edge.edge_name));
                }
            }
        }

        let r_edge_cache = &mut self.r_edge_cache;

        let mut mutations = Vec::with_capacity(edge_mutations.len());
        for (from, to, edge_name) in edge_mutations {
            let r_edge = match r_edge_cache.get(&edge_name.to_string()) {
                r_edge @ Some(_) => Ok(r_edge.map(String::from)),
                None => get_r_edge(&dynamodb, edge_name.clone()).await,
            };

            match r_edge {
                Ok(Some(r_edge)) if !r_edge.is_empty() => {
                    r_edge_cache.insert(edge_name.to_owned(), r_edge.to_string());
                    mutations.push(
                        upsert_edge(
                            &self.mg_client,
                            &mut self.metric_reporter,
                            &mut self.cache,
                            &to,
                            &from,
                            &r_edge,
                        )
                        .await,
                    )
                }
                Err(e) => {
                    error!("get_r_edge failed: {:?}", e);
                    edge_res = Some(e.to_string());
                }
                _ => warn!("Missing r_edge for f_edge {}", edge_name),
            }

            let upsert_res = upsert_edge(
                &self.mg_client,
                &mut self.metric_reporter,
                &mut self.cache,
                &from,
                &to,
                &edge_name,
            )
            .await;

            if let Err(e) = upsert_res {
                error!(
                    "Failed to upsert edge: {} {} {} {:?}",
                    &from, &to, &edge_name, e
                );
                edge_res = Some(format!("Failed to upsert edge: {:?}", e));
            }
        };

        identities
            .into_iter()
            .for_each(|identity| completed.add_identity(identity));

        completed
         */

        //unimplemented!()

        Err(Err(GraphMergerError::Unexpected("PLACEHOLDER".to_string())))
    }
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
