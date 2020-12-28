#![type_length_limit = "1195029"]

use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::io::{Cursor, Stdout};
use std::iter::FromIterator;
use std::net::ToSocketAddrs;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::UNIX_EPOCH;
use std::time::{Duration, SystemTime};

use async_trait::async_trait;
use aws_lambda_events::event::s3::{
    S3Bucket, S3Entity, S3Event, S3EventRecord, S3Object, S3RequestParameters, S3UserIdentity,
};
use aws_lambda_events::event::sqs::SqsEvent;
use chrono::Utc;
use dgraph_tonic::{Client as DgraphClient, Mutate, Query};
use failure::{bail, Error};
use futures::future::join_all;
use log::{debug, error, info, warn};
use prost::Message;
use rand::seq::SliceRandom;
use rand::thread_rng;
use rusoto_core::{HttpClient, Region};
use rusoto_dynamodb::AttributeValue;
use rusoto_dynamodb::DynamoDbClient;
use rusoto_dynamodb::{DynamoDb, GetItemInput};
use rusoto_s3::S3Client;
use rusoto_sqs::{SendMessageRequest, Sqs, SqsClient};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqs_executor::cache::{Cache, CacheResponse, Cacheable};
use sqs_executor::completion_event_serializer::CompletionEventSerializer;
use sqs_executor::event_decoder::PayloadDecoder;
use sqs_executor::event_handler::{EventHandler, CompletedEvents};
use sqs_executor::redis_cache::RedisCache;

use grapl_graph_descriptions::graph_description::{GeneratedSubgraphs, Graph, Node};
use grapl_graph_descriptions::node::NodeT;
use grapl_observe::dgraph_reporter::DgraphMetricReporter;
use grapl_observe::metric_reporter::MetricReporter;
use sqs_executor::errors::{CheckedError, Recoverable};
use std::convert::TryInto;
use sqs_executor::event_retriever::S3PayloadRetriever;
use sqs_executor::s3_event_emitter::S3EventEmitter;

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
    node_key: &str,
) -> Result<Option<String>, Error> {
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

    let query_res = txn
        .query_with_vars(QUERY, vars)
        .await
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

    Ok(uid)
}

async fn upsert_node(
    dg: &DgraphClient,
    metric_reporter: &mut MetricReporter<Stdout>,
    node: Node,
) -> Result<String, Error> {
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
    set_json["last_index_time"] = (SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Something is very wrong with the system clock")
        .as_millis() as u64)
        .into();

    let mut mu = dgraph_tonic::Mutation::new();
    mu.commit_now = true;
    mu.set_set_json(&set_json);

    let mut txn = dg.new_mutated_txn();
    let upsert_res = match txn.upsert(query, mu).await {
        Ok(res) => res,
        Err(e) => {
            txn.discard().await.map_err(AnyhowFailure::into_failure)?;
            return Err(e.into_failure().into());
        }
    };

    metric_reporter
        .mutation(&upsert_res, &[])
        .unwrap_or_else(|e| error!("mutation metric failed: {}", e));
    txn.commit().await.map_err(AnyhowFailure::into_failure)?;

    info!(
        "Upsert res for {}, set_json: {} upsert_res: {:?}",
        node_key,
        set_json.to_string(),
        upsert_res,
    );

    match node_key_to_uid(dg, metric_reporter, &node_key).await? {
        Some(uid) => Ok(uid),
        None => bail!("Could not retrieve uid after upsert for {}", &node_key),
    }
}

fn chunk<T, U>(data: U, count: usize) -> Vec<U>
where
    U: IntoIterator<Item = T>,
    U: FromIterator<T>,
    <U as IntoIterator>::IntoIter: ExactSizeIterator,
{
    let mut iter = data.into_iter();
    let iter = iter.by_ref();

    let chunk_len = (iter.len() / count) as usize + 1;

    let mut chunks = Vec::new();
    for _ in 0..count {
        chunks.push(iter.take(chunk_len).collect())
    }
    chunks
}

#[derive(Clone)]
struct GraphMerger<CacheT>
where
    CacheT: Cache + Clone + Send + Sync + 'static,
{
    mg_client: Arc<DgraphClient>,
    metric_reporter: MetricReporter<Stdout>,
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
            cache,
        }
    }
}

async fn upsert_edge(
    mg_client: &DgraphClient,
    metric_reporter: &mut MetricReporter<Stdout>,
    mu: dgraph_tonic::Mutation,
) -> Result<(), failure::Error> {
    let mut txn = mg_client.new_mutated_txn();
    let mut_res = txn
        .mutate_and_commit_now(mu)
        .await
        .map_err(AnyhowFailure::into_failure)?;
    metric_reporter
        .mutation(&mut_res, &[])
        .unwrap_or_else(|e| error!("edge mutation metric failed: {}", e));

    Ok(())
}

#[derive(Debug, Clone, Default)]
pub struct ZstdProtoDecoder;

impl<E> PayloadDecoder<E> for ZstdProtoDecoder
where
    E: Message + Default,
{
    fn decode(&mut self, body: Vec<u8>) -> Result<E, Box<dyn std::error::Error>>
    where
        E: Message + Default,
    {
        let mut decompressed = Vec::new();

        let mut body = Cursor::new(&body);

        zstd::stream::copy_decode(&mut body, &mut decompressed)?;

        Ok(E::decode(Cursor::new(decompressed))?)
    }
}

#[derive(Clone, Debug, Default)]
pub struct SubgraphSerializer {
    proto: Vec<u8>,
}

#[derive(thiserror::Error, Debug)]
pub enum SubgraphSerializerError {
    #[error("IO")]
    Io(#[from] std::io::Error),
    #[error("EncodeError")]
    EncodeError(#[from] prost::EncodeError),
}
impl CompletionEventSerializer for SubgraphSerializer {
    type CompletedEvent = GeneratedSubgraphs;
    type Output = Vec<u8>;
    type Error = SubgraphSerializerError;

    fn serialize_completed_events(
        &mut self,
        completed_events: &[Self::CompletedEvent],
    ) -> Result<Vec<Self::Output>, Self::Error> {
        if completed_events.is_empty() {
            warn!("No events to serialize");
            return Ok(Vec::new());
        }
        let mut subgraph = Graph::new(0);

        let mut pre_nodes = 0;
        let mut pre_edges = 0;
        for completed_event in completed_events {
            for sg in completed_event.subgraphs.iter() {
                pre_nodes += sg.nodes.len();
                pre_edges += sg.edges.len();
                subgraph.merge(sg);
            }
        }

        if subgraph.is_empty() {
            warn!(
                concat!(
                "Output subgraph is empty. Serializing to empty vector.",
                "pre_nodes: {} pre_edges: {}"
                ),
                pre_nodes, pre_edges,
            );
            return Ok(vec![]);
        }

        info!(
            "Serializing {} nodes {} edges. Down from {} nodes {} edges.",
            subgraph.nodes.len(),
            subgraph.edges.len(),
            pre_nodes,
            pre_edges,
        );

        let subgraphs = GeneratedSubgraphs {
            subgraphs: vec![subgraph],
        };

        self.proto.clear();

        prost::Message::encode(&subgraphs, &mut self.proto)?;

        let mut compressed = Vec::with_capacity(self.proto.len());
        let mut proto = Cursor::new(&self.proto);
        zstd::stream::copy_encode(&mut proto, &mut compressed, 4)?;
        Ok(vec![compressed])
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

fn map_sqs_message(event: aws_lambda_events::event::sqs::SqsMessage) -> rusoto_sqs::Message {
    rusoto_sqs::Message {
        attributes: Some(event.attributes),
        body: event.body,
        md5_of_body: event.md5_of_body,
        md5_of_message_attributes: event.md5_of_message_attributes,
        message_attributes: None,
        message_id: event.message_id,
        receipt_handle: event.receipt_handle,
    }
}

async fn handler() -> Result<(), Box<dyn std::error::Error>> {
    info!("Handling event");

    let source_queue_url = std::env::var("SOURCE_QUEUE_URL").expect("SOURCE_QUEUE_URL");
    debug!("Queue Url: {}", source_queue_url);
    let bucket_prefix = std::env::var("BUCKET_PREFIX").expect("BUCKET_PREFIX");


    let sqs_client = SqsClient::new(grapl_config::region());
    let s3_client = S3Client::new(grapl_config::region());

    let cache_address = {
        let cache_addr =
            std::env::var("MERGED_CACHE_ADDR").expect("MERGED_CACHE_ADDR");
        let cache_port =
            std::env::var("MERGED_CACHE_PORT").expect("MERGED_CACHE_PORT");

        format!(
            "{}:{}",
            cache_addr, cache_port,
        )
    };

    let destination_bucket = bucket_prefix + "-subgraphs-merged-bucket";

    let mut cache = Vec::with_capacity(10);
    for _ in 0..10u8 {
        let c = RedisCache::new(cache_address.to_owned())
            .await
            .expect("Could not create redis client");
        cache.push(c);
    }
    let mut cache: [_; 10] = cache.try_into().unwrap_or_else(|_| panic!("ahhh"));
    let cache = &mut cache;

    let serializer = vec![SubgraphSerializer::default(); 10];
    let mut serializer: [_; 10] = serializer.try_into().unwrap_or_else(|_| panic!("ahhh"));
    let mut serializer = &mut serializer;

    let mut s3_emitter = Vec::with_capacity(10);
    for _ in 0..10u8 {
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
        ZstdProtoDecoder::default(),
        MetricReporter::<Stdout>::new("graph-merger"),
    ); 10];

    let mut s3_payload_retriever: [_; 10] = s3_payload_retriever.try_into().unwrap_or_else(|_|panic!("ahhh"));
    let s3_payload_retriever = &mut s3_payload_retriever;

    info!("Output events to: {}", destination_bucket);

    let metric_reporter = MetricReporter::<Stdout>::new("graph-merger");

    let fake_generator = vec![
        GraphMerger::new(grapl_config::mg_alphas(), metric_reporter.clone(), cache[0].clone());
        10
    ];
    let mut fake_generator: [_; 10] = fake_generator.try_into().unwrap_or_else(|_|panic!("ahhh"));
    let mut fake_generator = &mut fake_generator;

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
    type OutputEvent = GeneratedSubgraphs;
    type Error = GraphMergerError;

    async fn handle_event(
        &mut self,
        generated_subgraphs: GeneratedSubgraphs,
        completed: &mut CompletedEvents,
    ) -> Result<Self::OutputEvent, Result<(Self::OutputEvent, Self::Error), Self::Error>> {
        let mut subgraph = Graph::new(0);
        for generated_subgraph in generated_subgraphs.subgraphs {
            subgraph.merge(&generated_subgraph);
        }

        if subgraph.is_empty() {
            warn!("Attempted to merge empty subgraph. Short circuiting.");
            return Ok(GeneratedSubgraphs { subgraphs: vec![] });
        }

        let mut identities = Vec::with_capacity(subgraph.nodes.len() + subgraph.edges.len());

        info!(
            "handling new subgraph with {} nodes {} edges",
            subgraph.nodes.len(),
            subgraph.edges.len(),
        );

        let mut upsert_res = None;
        let mut edge_res = None;

        let mut node_key_to_uid_map = HashMap::new();
        use futures::future::FutureExt;
        let mut upserts = Vec::with_capacity(subgraph.nodes.len());
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
                upsert_node(&self.mg_client, &mut self.metric_reporter, node.clone())
                    .map(move |u| (node.clone_node_key(), u))
                    .await,
            )
        }

        for (node_key, upsert) in upserts.into_iter() {
            let new_uid = match upsert {
                Ok(new_uid) => {
                    let identity = subgraph.nodes[&node_key].clone().into_json().to_string();
                    identities.push(identity);
                    new_uid
                }
                Err(e) => {
                    error!("{}", e);
                    upsert_res = Some(e);
                    continue;
                }
            };

            node_key_to_uid_map.insert(node_key, new_uid);
        }

        if node_key_to_uid_map.is_empty() && upsert_res.is_some() {
            return Err(Err(
                GraphMergerError::Unexpected(format!(
                    "All nodes failed to upsert {:?}",
                    upsert_res
                )),
            ));
        }

        info!("Upserted: {} nodes", node_key_to_uid_map.len());

        info!("Inserting edges {}", subgraph.edges.len());
        let dynamodb = init_dynamodb_client();

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
                    match node_key_to_uid(&self.mg_client, &mut self.metric_reporter, &edge.to[..])
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

        let mut r_edge_cache: HashMap<String, String> = HashMap::with_capacity(2);

        let mut mutations = Vec::with_capacity(edge_mutations.len());
        for (from, to, edge_name) in edge_mutations {
            let r_edge = match r_edge_cache.get(&edge_name.to_string()) {
                r_edge @ Some(_) => Ok(r_edge.map(String::from)),
                None => get_r_edge(&dynamodb, edge_name.clone()).await,
            };

            match r_edge {
                Ok(Some(r_edge)) if !r_edge.is_empty() => {
                    r_edge_cache.insert(edge_name.to_owned(), r_edge.to_string());
                    let mu = generate_edge_insert(&to, &from, &r_edge);
                    mutations
                        .push(upsert_edge(&self.mg_client, &mut self.metric_reporter, mu).await)
                }
                Err(e) => {
                    error!("get_r_edge failed: {:?}", e);
                    edge_res = Some(e.to_string());
                }
                _ => warn!("Missing r_edge for f_edge {}", edge_name),
            }

            let mu = generate_edge_insert(&from, &to, &edge_name);
            mutations.push(upsert_edge(&self.mg_client, &mut self.metric_reporter, mu).await);
        }

        if let Some(e) = mutations.iter().find(|e| e.is_err()) {
            error!("Failed to upsert edge: {:?}", e);
            edge_res = Some(format!("Failed to upsert edge: {:?}", e));
        }

        identities
            .into_iter()
            .for_each(|identity| completed.add_identity(identity));

        match (upsert_res, edge_res) {
            (Some(e), _) => Err(Ok((
                GeneratedSubgraphs::new(vec![subgraph]),
                GraphMergerError::Unexpected(e.to_string()),
            ))),
            (_, Some(e)) => Err(Ok((
                GeneratedSubgraphs::new(vec![subgraph]),
                GraphMergerError::Unexpected(e.to_string()),
            ))),
            (None, None) => {
                Ok(GeneratedSubgraphs::new(vec![subgraph]))
            }
        }
    }
}

pub fn init_dynamodb_client() -> DynamoDbClient {
    if grapl_config::is_local() {
        info!("Connecting to local DynamoDB http://dynamodb:8000");
        DynamoDbClient::new_with(
            HttpClient::new().expect("failed to create request dispatcher"),
            rusoto_credential::StaticProvider::new_minimal(
                "dummy_cred_aws_access_key_id".to_owned(),
                "dummy_cred_aws_secret_access_key".to_owned(),
            ),
            Region::Custom {
                name: "us-west-2".to_string(),
                endpoint: "http://dynamodb:8000".to_string(),
            },
        )
    } else {
        info!("Connecting to DynamoDB");
        let region = grapl_config::region();
        DynamoDbClient::new(region.clone())
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

    let item = client
        .get_item(query)
        .await
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

fn init_sqs_client() -> SqsClient {
    info!("Connecting to local us-east-1 http://sqs.us-east-1.amazonaws.com:9324");

    SqsClient::new_with(
        HttpClient::new().expect("failed to create request dispatcher"),
        rusoto_credential::StaticProvider::new_minimal(
            "dummy_sqs".to_owned(),
            "dummy_sqs".to_owned(),
        ),
        Region::Custom {
            name: "us-east-1".to_string(),
            endpoint: "http://sqs.us-east-1.amazonaws.com:9324".to_string(),
        },
    )
}

fn init_s3_client() -> S3Client {
    info!("Connecting to local http://s3:9000");
    S3Client::new_with(
        HttpClient::new().expect("failed to create request dispatcher"),
        rusoto_credential::StaticProvider::new_minimal(
            "minioadmin".to_owned(),
            "minioadmin".to_owned(),
        ),
        Region::Custom {
            name: "locals3".to_string(),
            endpoint: "http://s3:9000".to_string(),
        },
    )
}

#[derive(Clone, Default)]
struct HashCache {
    cache: Arc<Mutex<std::collections::HashSet<Vec<u8>>>>,
}

#[derive(thiserror::Error, Debug)]
pub enum HashCacheError {
    // standin until the never type (`!`) is stable
    #[error("HashCacheError.Unreachable")]
    Unreachable
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
    let env = grapl_config::init_grapl_env!();
    info!("Starting graph-merger 0");
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
