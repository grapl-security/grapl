extern crate aws_lambda_events;
extern crate base16;
extern crate base64;
extern crate dgraph_rs;
#[macro_use]
extern crate failure;
extern crate futures;
extern crate graph_descriptions;
extern crate grpc;
extern crate itertools;
extern crate lambda_runtime as lambda;
#[macro_use]
extern crate log;

extern crate prost;
extern crate rand;
extern crate rusoto_core;
extern crate rusoto_s3;
extern crate rusoto_sns;
extern crate rusoto_sqs;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate sha2;
extern crate simple_logger;
extern crate sqs_lambda;
extern crate stopwatch;

use std::collections::{HashMap, HashSet};
use std::io::Cursor;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use aws_lambda_events::event::sqs::{SqsEvent, SqsMessage};
use dgraph_rs::DgraphClient;
use dgraph_rs::protos::api;
use dgraph_rs::protos::api_grpc;
use failure::Error;
use futures::Future;
use futures::future::join_all;

use graph_descriptions::node::NodeT;
use graph_descriptions::graph_description::GeneratedSubgraphs;
use graph_descriptions::graph_description::{Graph, Node};
use graph_descriptions::graph_description::node::WhichNode;
use graph_descriptions::process::ProcessState;
use graph_descriptions::file::FileState;
use graph_descriptions::process_inbound_connection::ProcessInboundConnectionState;
use graph_descriptions::process_outbound_connection::ProcessOutboundConnectionState;
use graph_descriptions::network_connection::NetworkConnectionState;

use grpc::{Client, ClientStub};
use grpc::ClientConf;
use itertools::Itertools;
use lambda::Context;
use lambda::error::HandlerError;
use lambda::lambda;
use prost::Message;
use rand::seq::SliceRandom;
use rand::thread_rng;
use rusoto_core::{HttpClient, Region};
use rusoto_s3::{S3, S3Client};
use rusoto_sns::{Sns, SnsClient};
use rusoto_sns::PublishInput;
use rusoto_sqs::{GetQueueUrlRequest, Sqs, SqsClient, SendMessageRequest};
use sha2::{Digest, Sha256};
use sqs_lambda::completion_event_serializer::CompletionEventSerializer;
use sqs_lambda::event_decoder::PayloadDecoder;
use sqs_lambda::s3_event_emitter::S3EventEmitter;
use sqs_lambda::event_handler::{EventHandler, OutputEvent, Completion};

use sqs_lambda::event_processor::{EventProcessor, EventProcessorActor};
use sqs_lambda::event_retriever::S3PayloadRetriever;

use sqs_lambda::sqs_completion_handler::{CompletionPolicy, SqsCompletionHandler, SqsCompletionHandlerActor};
use sqs_lambda::sqs_consumer::{ConsumePolicy, SqsConsumer, SqsConsumerActor};

use async_trait::async_trait;



use sqs_lambda::redis_cache::RedisCache;


use crate::futures::FutureExt;
use serde_json::Value;
use std::iter::FromIterator;
use sqs_lambda::cache::{CacheResponse, Cache, NopCache};
use std::fmt::Debug;
use sqs_lambda::local_service::local_service;
use aws_lambda_events::event::s3::{S3Object, S3UserIdentity, S3Bucket, S3Entity, S3RequestParameters, S3Event, S3EventRecord};
use sqs_lambda::local_sqs_service::local_sqs_service;
use chrono::Utc;
use tokio::runtime::Runtime;

macro_rules! log_time {
    ($msg:expr, $x:expr) => {
        {
            let mut sw = stopwatch::Stopwatch::start_new();
            #[allow(path_statements)]
            let result = $x;
            sw.stop();
            info!("{} {} milliseconds", $msg, sw.elapsed_ms());
            result
        }
    };
}

fn generate_edge_insert(from: &str, to: &str, edge_name: &str) -> api::Mutation {
    let mu = json!({
        "uid": from,
        edge_name: {
            "uid": to
        }
    }).to_string().into_bytes();

    let mut mutation = api::Mutation::new();
    mutation.commit_now = true;
    mutation.set_json = mu;
    mutation
}

async fn node_key_to_uid(dg: &DgraphClient, node_key: &str) -> Result<Option<String>, Error> {

    let mut txn = dg.new_read_only();

    const QUERY: & str = r"
       query q0($a: string)
    {
        q0(func: eq(node_key, $a), first: 1) {
            uid
        }
    }
    ";

    let mut vars = HashMap::new();
    vars.insert("$a".to_string(), node_key.into());

    let query_res: Value = txn.query_with_vars(QUERY, vars).await
        .map(|res| serde_json::from_slice(&res.json))??;

    let uid = query_res.get("q0")
        .and_then(|res| res.get(0))
        .and_then(|uid| uid.get("uid"))
        .and_then(|uid| uid.as_str())
        .map(String::from);

    Ok(uid)
}

async fn upsert_node(dg: &DgraphClient, node: Node) -> Result<String, Error> {
    let node_key = node.clone_node_key();
    let query = format!(r#"
                {{
                  p as var(func: eq(node_key, "{}"), first: 1)
                }}
                "#, node.get_node_key());

    let node_key = node.clone_node_key();
    let mut set_json = node.into_json();
    set_json["uid"] = "uid(p)".into();


    let mu = api::Mutation {
        set_json: set_json.to_string().into_bytes(),
        commit_now: true,
        ..Default::default()
    };

    let mut txn = dg.new_txn();
    let upsert_res = txn.upsert(
        query, mu,
    )
        .await
        .expect(&format!("Request to dgraph failed {:?}", &node_key));

    txn.commit_or_abort().await?;

    info!(
        "Upsert res for {}, set_json: {} upsert_res: {:?}",
        node_key,
        set_json.to_string(),
        upsert_res,
    );

    match node_key_to_uid(dg, &node_key).await? {
        Some(uid) => {
            Ok(uid)
        },
        None => bail!("Could not retrieve uid after upsert for {}", &node_key),
    }

//    if let Some(uid) = upsert_res.uids.values().next() {
//        Ok(uid.to_owned())
//    } else {
//        match node_key_to_uid(dg, &node_key).await? {
//            Some(uid) => {
//                Ok(uid)
//            },
//            None => bail!("Could not retrieve uid after upsert for {}", &node_key),
//        }
//    }
}


fn chunk<T, U>(data: U, count: usize) -> Vec<U>
    where U: IntoIterator<Item=T>,
          U: FromIterator<T>,
          <U as IntoIterator>::IntoIter: ExactSizeIterator
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
struct GraphMerger<CacheT, CacheErr>
    where
        CacheT: Cache<CacheErr> + Clone + Send + Sync + 'static,
        CacheErr: Debug + Clone + Send + Sync + 'static,
{
    mg_alphas: Vec<String>,
    cache: CacheT,
    _p: std::marker::PhantomData<CacheErr>
}

impl<CacheT, CacheErr> GraphMerger<CacheT, CacheErr>
    where
        CacheT: Cache<CacheErr> + Clone + Send + Sync + 'static,
        CacheErr: Debug + Clone + Send + Sync + 'static,
{
    pub fn new(mg_alphas: Vec<String>, cache: CacheT) -> Self {
        Self {
            mg_alphas,
            cache,
            _p: std::marker::PhantomData
        }
    }
}

async fn upsert_edge(mg_client: &DgraphClient, mu: api::Mutation) -> Result<(), Error> {
    let mut txn = mg_client.new_txn();
    let upsert_res = txn.mutate(mu).await?;

    txn.commit_or_abort().await?;

    Ok(())
}


#[derive(Debug, Clone, Default)]
pub struct ZstdProtoDecoder;

impl<E> PayloadDecoder<E> for ZstdProtoDecoder
    where E: Message + Default
{
    fn decode(&mut self, body: Vec<u8>) -> Result<E, Box<dyn std::error::Error>>
        where E: Message + Default,
    {
        let mut decompressed = Vec::new();

        let mut body = Cursor::new(&body);

        zstd::stream::copy_decode(&mut body, &mut decompressed)?;

        Ok(E::decode(decompressed)?)
    }
}

#[derive(Clone, Debug, Default)]
pub struct SubgraphSerializer {
    proto: Vec<u8>,
}

impl CompletionEventSerializer for SubgraphSerializer {
    type CompletedEvent = GeneratedSubgraphs;
    type Output = Vec<u8>;
    type Error = sqs_lambda::error::Error<Arc<failure::Error>>;

    fn serialize_completed_events(
        &mut self,
        completed_events: &[Self::CompletedEvent],
    ) -> Result<Vec<Self::Output>, Self::Error> {
        let mut subgraph = Graph::new(
            0
        );

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
                pre_nodes,
                pre_edges,
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

        let subgraphs = GeneratedSubgraphs { subgraphs: vec![subgraph] };

        self.proto.clear();

        prost::Message::encode(&subgraphs, &mut self.proto)
            .map(Arc::new)
            .map_err(|e| {
                sqs_lambda::error::Error::EncodeError(e.to_string())
            })?;

        let mut compressed = Vec::with_capacity(self.proto.len());
        let mut proto = Cursor::new(&self.proto);
        zstd::stream::copy_encode(&mut proto, &mut compressed, 4)
            .map(Arc::new)
            .map_err(|e| {
                sqs_lambda::error::Error::EncodeError(e.to_string())
            })?;

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

    format!(
        "{}/{}-{}",
        cur_day, cur_ms, uuid::Uuid::new_v4()
    )
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

fn handler(event: SqsEvent, ctx: Context) -> Result<(), HandlerError> {
    info!("Handling event");

    let mut initial_events: HashSet<String> = event.records
        .iter()
        .map(|event| event.message_id.clone().unwrap())
        .collect();

    info!("Initial Events {:?}", initial_events);

    let (tx, rx) = std::sync::mpsc::sync_channel(10);
    let completed_tx = tx.clone();

    std::thread::spawn(move || {
        tokio_compat::run_std(
            async move {
                let queue_url = std::env::var("QUEUE_URL").expect("QUEUE_URL");
                info!("Queue Url: {}", queue_url);
                let bucket_prefix = std::env::var("BUCKET_PREFIX").expect("BUCKET_PREFIX");
                let cache_address = {
                    let retry_identity_cache_addr = std::env::var("MERGED_CACHE_ADDR").expect("MERGED_CACHE_ADDR");
                    let retry_identity_cache_port = std::env::var("MERGED_CACHE_PORT").expect("MERGED_CACHE_PORT");

                    format!(
                        "{}:{}",
                        retry_identity_cache_addr,
                        retry_identity_cache_port,
                    )
                };

                let bucket = std::env::var("SUBGRAPH_MERGED_BUCKET").expect("SUBGRAPH_MERGED_BUCKET");
                info!("Output events to: {}", bucket);
                let region = {
                    let region_str = std::env::var("AWS_REGION").expect("AWS_REGION");
                    Region::from_str(&region_str).expect("Region error")
                };
                let mg_alphas: Vec<_> = std::env::var("MG_ALPHAS").expect("MG_ALPHAS")
                    .split(',')
                    .map(str::to_string)
                    .collect();

                let cache = RedisCache::new(cache_address.to_owned()).await.expect("Could not create redis client");

                let graph_merger
                    : GraphMerger<_, sqs_lambda::error::Error<Arc<failure::Error>>>
                    = GraphMerger::new(
                        mg_alphas,
                        cache.clone(),
                );

                let initial_messages: Vec<_> = event.records
                    .into_iter()
                    .map(map_sqs_message)
                    .collect();

                sqs_lambda::sqs_service::sqs_service(
                    queue_url,
                    initial_messages,
                    bucket,
                    ctx,
                    S3Client::new(region.clone()),
                    SqsClient::new(region.clone()),
                    ZstdProtoDecoder::default(),
                    SubgraphSerializer { proto: Vec::with_capacity(1024) },
                    graph_merger,
                    cache.clone(),
                    move |_self_actor, result: Result<String, String>| {
                        match result {
                            Ok(worked) => {
                                info!("Handled an event, which was successfully deleted: {}", &worked);
                                tx.send(worked).unwrap();
                            }
                            Err(worked) => {
                                info!("Handled an initial_event, though we failed to delete it: {}", &worked);
                                tx.send(worked).unwrap();
                            }
                        }
                    },
                    move |_, _| async move {Ok(())},
                ).await;
                completed_tx.clone().send("Completed".to_owned()).unwrap();
            });
    });

    info!("Checking acks");
    for r in &rx {
        info!("Acking event: {}", &r);
        initial_events.remove(&r);
        if r == "Completed" {
            // If we're done go ahead and try to clear out any remaining
            while let Ok(r) = rx.try_recv() {
                initial_events.remove(&r);
            }
            break;
        }
    }

    info!("Completed execution");

    if initial_events.is_empty() {
        info!("Successfully acked all initial events");
        Ok(())
    } else {
        Err(lambda::error::HandlerError::from("Failed to ack all initial events"))
    }
}

#[async_trait]
impl<CacheT, CacheErr> EventHandler for GraphMerger<CacheT, CacheErr>
    where
        CacheT: Cache<CacheErr> + Clone + Send + Sync + 'static,
        CacheErr: Debug + Clone + Send + Sync + 'static,
{
    type InputEvent = GeneratedSubgraphs;
    type OutputEvent = GeneratedSubgraphs;
    type Error = sqs_lambda::error::Error<Arc<failure::Error>>;

    async fn handle_event(&mut self, generated_subgraphs: GeneratedSubgraphs) -> OutputEvent<Self::OutputEvent, Self::Error> {
        let mut subgraph = Graph::new(0);
        for generated_subgraph in generated_subgraphs.subgraphs {
            subgraph.merge(&generated_subgraph);
        }

        if subgraph.is_empty() {
            warn!("Attempted to merge empty subgraph. Short circuiting.");
            return OutputEvent::new(Completion::Total(
                GeneratedSubgraphs {subgraphs: vec![]}
            ))
        }

        println!("handling new subgraph with {} nodes {} edges", subgraph.nodes.len(), subgraph.edges.len());

        let mg_client = {
            let mut rng = thread_rng();
            let rand_alpha = self.mg_alphas.choose(&mut rng)
                .expect("Empty rand_alpha");

            DgraphClient::new(
                vec![
                    api_grpc::DgraphClient::with_client(
                        Arc::new(
                            Client::new_plain(rand_alpha, 9080, ClientConf {
                                ..Default::default()
                            }).expect("Failed to create dgraph client")
                        )
                    )
                ]
            )

        };

//        async_handler(mg_client, subgraph).await;

        let mut upsert_res = None;
        let mut edge_res = None;

        let mut node_key_to_uid = HashMap::new();

        let upserts = subgraph.nodes.values().map(|node| {
            upsert_node(&mg_client, node.clone()).map(move |u| (node.get_node_key(), u))
        });

        let upserts = log_time!("All upserts", join_all(upserts).await);

        for (node_key, upsert) in upserts {
            let new_uid = match upsert  {
                Ok(new_uid) => new_uid,
                Err(e) => {  warn!("{}", e); upsert_res = Some(e); continue}
            };

            node_key_to_uid.insert(node_key, new_uid);
        }

        if node_key_to_uid.is_empty() {
            return OutputEvent::new(Completion::Error(
                sqs_lambda::error::Error::ProcessingError(
                    Arc::new(
                        (|| {
                            bail!("All nodes failed to identify");
                            Ok(())
                        })().unwrap_err()
                    )
                ))
            );
        }

        info!("Upserted: {} nodes", node_key_to_uid.len());

        info!("Inserting edges {}", subgraph.edges.len());

        let edge_mutations: Vec<_> = subgraph.edges
            .values()
            .map(|e| &e.edges)
            .flatten()
            .filter_map(|edge| {
                match (node_key_to_uid.get(&edge.from[..]), node_key_to_uid.get(&edge.to[..])) {
                    (Some(from), Some(to)) if from == to => {
                        let err =
                            format!(
                                "From and To can not be the same uid {} {} {} {} {}",
                                from,
                                to,
                                &edge.from[..],
                                &edge.to[..],
                                &edge.edge_name);
                        error!("{}", err);
                        edge_res = Some(
                            err
                        );
                        None
                    }
                    (Some(from), Some(to)) => {
                        info!("Upserting edge: {} {} {}", &from, &to, &edge.edge_name);
                        Some(generate_edge_insert(&from, &to, &edge.edge_name))
                    }
                    (_, _) => {
                        edge_res = Some("Edge to uid failed".to_string()); None
                    }
                }
            })
            .map(|mu| upsert_edge(&mg_client, mu))
            .collect();

        let _: Vec<_> = join_all(edge_mutations).await;

//        let identities: Vec<_> = unid_id_map.keys().cloned().collect();

        let mut completed = match (upsert_res, edge_res) {
            (Some(e), _) => {
                OutputEvent::new(
                    Completion::Partial(
                        (
                            GeneratedSubgraphs::new(vec![subgraph]),
                            sqs_lambda::error::Error::ProcessingError(Arc::new(
                                e
                            ))
                        )
                    )
                )
            }
            (_, Some(e)) => {
                OutputEvent::new(
                    Completion::Partial(
                        (
                            GeneratedSubgraphs::new(vec![subgraph]),
                            sqs_lambda::error::Error::ProcessingError(
                                Arc::new(
                                    (
                                        || {
                                            bail!("{}", e);
                                            Ok(())
                                        }
                                    )().unwrap_err()
                                )
                            )
                        )
                    )
                )
            }
            (None, None) => OutputEvent::new(Completion::Total(GeneratedSubgraphs::new(vec![subgraph])))
        };


//        identities.into_iter().for_each(|identity| completed.add_identity(identity));

        completed

    }
}


fn init_sqs_client() -> SqsClient
{
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
        }
    )
}

fn init_s3_client() -> S3Client
{
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

async fn inner_main() -> Result<(), Box<dyn std::error::Error>> {
    let mg_alphas: Vec<_> = vec!["master_graph".to_owned()];

    let graph_merger
        : GraphMerger<_, sqs_lambda::error::Error<Arc<failure::Error>>>
        =
        GraphMerger::new(mg_alphas, NopCache {});

    local_sqs_service(
        "http://sqs.us-east-1.amazonaws.com:9324/queue/graph-merger-queue",
        "local-grapl-subgraphs-merged-bucket",
        Context {
            deadline: Utc::now().timestamp_millis() + 10_000,
            ..Default::default()
        },
        init_s3_client(),
        init_sqs_client(),
        ZstdProtoDecoder::default(),
        SubgraphSerializer { proto: Vec::with_capacity(1024) },
        graph_merger,
        NopCache {},
        |_, event_result | {dbg!(event_result);},
        move |bucket, key| async move {
            let output_event = S3Event {
                records: vec![
                    S3EventRecord {
                        event_version: None,
                        event_source: None,
                        aws_region: None,
                        event_time: chrono::Utc::now(),
                        event_name: None,
                        principal_id: S3UserIdentity { principal_id: None },
                        request_parameters: S3RequestParameters { source_ip_address: None },
                        response_elements: Default::default(),
                        s3: S3Entity {
                            schema_version: None,
                            configuration_id: None,
                            bucket: S3Bucket {
                                name: Some(bucket),
                                owner_identity: S3UserIdentity { principal_id: None },
                                arn: None
                            },
                            object: S3Object {
                                key: Some(key),
                                size: 0,
                                url_decoded_key: None,
                                version_id: None,
                                e_tag: None,
                                sequencer: None
                            }
                        }
                    }
                ]
            };

            let sqs_client = init_sqs_client();

            // publish to SQS
            sqs_client.send_message(
                SendMessageRequest {
                    message_body: serde_json::to_string(&output_event)
                        .expect("failed to encode s3 event"),
                    queue_url: "http://sqs.us-east-1.amazonaws.com:9324/queue/analyzer-dispatcher-queue".to_string(),
                    ..Default::default()
                }
            ).await?;

            Ok(())
        }
    ).await;


    Ok(())

}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    simple_logger::init_with_level(log::Level::Info).unwrap();

    let is_local = std::env::var("IS_LOCAL")
        .is_ok();

    if is_local {
        info!("Running locally");
        std::thread::sleep_ms(10_000);

        let mut runtime = Runtime::new().unwrap();

        loop {
            if let Err(e) = runtime.block_on(async move { inner_main().await }) {
                error!("{}", e);
            }

            std::thread::sleep_ms(2_000);
        }
    }  else {
        info!("Running in AWS");
        lambda!(handler);
    }

    Ok(())
}
