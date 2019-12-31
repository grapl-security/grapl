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
use rusoto_core::Region;
use rusoto_s3::{S3, S3Client};
use rusoto_sns::{Sns, SnsClient};
use rusoto_sns::PublishInput;
use rusoto_sqs::{GetQueueUrlRequest, Sqs, SqsClient};
use sha2::{Digest, Sha256};
use sqs_lambda::completion_event_serializer::CompletionEventSerializer;
use sqs_lambda::event_decoder::PayloadDecoder;
use sqs_lambda::event_emitter::S3EventEmitter;
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
use sqs_lambda::cache::CacheResponse;

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

    info!("Upsert res for {}, {}: {:?}", node_key, set_json.to_string(), upsert_res);

    if let Some(uid) = upsert_res.uids.values().next() {
        Ok(uid.to_owned())
    } else {
        match node_key_to_uid(dg, &node_key).await? {
            Some(uid) => {
                Ok(uid)
            },
            None => bail!("Could not retrieve uid after upsert for &node_key"),
        }
    }
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

////pub fn subgraph_to_sns<S>(sns_client: &S, mut subgraphs: Graph) -> Result<(), Error>
////    where S: Sns
////{
////    let mut proto = Vec::with_capacity(8192);
////    let mut compressed = Vec::with_capacity(proto.len());
////
////    for nodes in chunk(subgraphs.nodes, 1000) {
////        proto.clear();
////        compressed.clear();
////        let mut edges = HashMap::new();
////        for node in nodes.keys() {
////            let node_edges = subgraphs.edges.remove(node);
////            if let Some(node_edges) = node_edges {
////                edges.insert(node.to_owned(), node_edges);
////            }
////        }
////        let subgraph = Graph {
////            nodes,
////            edges,
////            timestamp: subgraphs.timestamp
////        };
////
////        subgraph.encode(&mut proto)?;
////
////        let subgraph_merged_topic_arn = std::env::var("SUBGRAPH_MERGED_TOPIC_ARN").expect("SUBGRAPH_MERGED_TOPIC_ARN");
////
////        let mut proto = Cursor::new(&proto);
////        zstd::stream::copy_encode(&mut proto, &mut compressed, 4)
////            .expect("compress zstd capnp");
////
////        let message = base64::encode(&compressed);
////
////        info!("Message is {} bytes", message.len());
////
////        sns_client.publish(
////            PublishInput {
////                message,
////                topic_arn: subgraph_merged_topic_arn.into(),
////                ..Default::default()
////            }
////        )
////            .with_timeout(Duration::from_secs(5))
////            .sync()?;
////    }
//
//    // If we still have edges, but the nodes were not part of the subgraph, emit those as another event
//    if !subgraphs.edges.is_empty() {
//        for edges in chunk(subgraphs.edges, 1000) {
//            proto.clear();
//            compressed.clear();
//            let subgraph = Graph {
//                nodes: HashMap::new(),
//                edges,
//                timestamp: subgraphs.timestamp
//            };
//
//            subgraph.encode(&mut proto)?;
//
//            let subgraph_merged_topic_arn = std::env::var("SUBGRAPH_MERGED_TOPIC_ARN").expect("SUBGRAPH_MERGED_TOPIC_ARN");
//
//            let mut proto = Cursor::new(&proto);
//            zstd::stream::copy_encode(&mut proto, &mut compressed, 4)
//                .expect("compress zstd capnp");
//
//            let message = base64::encode(&compressed);
//
//            info!("Message is {} bytes", message.len());
//
//            sns_client.publish(
//                PublishInput {
//                    message,
//                    topic_arn: subgraph_merged_topic_arn.into(),
//                    ..Default::default()
//                }
//            )
//                .with_timeout(Duration::from_secs(5))
//                .sync()?;
//
//        }
//    }
//
//    Ok(())
//}


#[derive(Clone)]
struct GraphMerger {
    mg_alphas: Vec<String>,
    cache: RedisCache,
}

impl GraphMerger {
    pub fn new(mg_alphas: Vec<String>, cache: RedisCache) -> Self {
        Self {
            mg_alphas,
            cache
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
    type Error = failure::Error;

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

                let node_identifier = GraphMerger::new(
                    mg_alphas,
                    cache.clone(),
                );

                info!("SqsCompletionHandler");

                let finished_tx = tx.clone();
                let sqs_completion_handler = SqsCompletionHandlerActor::new(
                    SqsCompletionHandler::new(
                        SqsClient::new(region.clone()),
                        queue_url.to_string(),
                        SubgraphSerializer { proto: Vec::with_capacity(1024) },
                        S3EventEmitter::new(
                            S3Client::new(region.clone()),
                            bucket.to_owned(),
                            time_based_key_fn,
                        ),
                        CompletionPolicy::new(
                            1000, // Buffer up to 1000 messages
                            Duration::from_secs(30), // Buffer for up to 30 seconds
                        ),
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
                    )
                );

                info!("Defining consume policy");
                let consume_policy = ConsumePolicy::new(
                    ctx, // Use the Context.deadline from the lambda_runtime
                    Duration::from_secs(10), // Stop consuming when there's 2 seconds left in the runtime
                    3, // If we get 3 empty receives in a row, stop consuming
                );

                info!("Defining consume policy");
                let (shutdown_tx, shutdown_notify) = tokio::sync::oneshot::channel();

                info!("SqsConsumer");
                let sqs_consumer = SqsConsumerActor::new(
                    SqsConsumer::new(
                        SqsClient::new(region.clone()),
                        queue_url.clone(),
                        consume_policy,
                        sqs_completion_handler.clone(),
                        shutdown_tx,
                    )
                );

                info!("EventProcessors");
                let event_processors: Vec<_> = (0..10)
                    .map(|_| {
                        EventProcessorActor::new(EventProcessor::new(
                            sqs_consumer.clone(),
                            sqs_completion_handler.clone(),
                            node_identifier.clone(),
                            S3PayloadRetriever::new(S3Client::new(region.clone()), ZstdProtoDecoder::default()),
                            cache.clone(),
                        ))
                    })
                    .collect();

                info!("Start Processing");

                futures::future::join_all(event_processors.iter().map(|ep| ep.start_processing())).await;

                let mut proc_iter = event_processors.iter().cycle();
                for event in event.records {
                    let next_proc = proc_iter.next().unwrap();
                    next_proc.process_event(
                        map_sqs_message(event)
                    ).await;
                }

                info!("Waiting for shutdown notification");

                // Wait for the consumers to shutdown
                let _ = shutdown_notify.await;
                info!("Consumer shutdown");
                finished_tx.send("Completed".to_owned()).unwrap();
            });
    });

    info!("Checking acks");
    for r in &rx {
        info!("Acking event: {}", &r);
        initial_events.remove(&r);
        if r == "Completed" {
            let r = rx.recv_timeout(Duration::from_millis(100));
            if let Ok(r) = r {
                initial_events.remove(&r);
            }
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
impl EventHandler for GraphMerger
{
    type InputEvent = GeneratedSubgraphs;
    type OutputEvent = GeneratedSubgraphs;
    type Error = Arc<failure::Error>;

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
            return OutputEvent::new(
                Completion::Error(
                    Arc::new(
                        (
                            || {
                                bail!("Failed to attribute uids to any of {} node keys", subgraph.nodes.len());
                                Ok(())
                            }
                        )().unwrap_err()
                    )
                )
            )
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
                        (GeneratedSubgraphs::new(vec![subgraph]), Arc::new(
                            e
                        ))
                    )
                )
            }
            (_, Some(e)) => {
                OutputEvent::new(
                    Completion::Partial(
                        (
                            GeneratedSubgraphs::new(vec![subgraph]),
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
            }
            (None, None) => OutputEvent::new(Completion::Total(GeneratedSubgraphs::new(vec![subgraph])))
        };


//        identities.into_iter().for_each(|identity| completed.add_identity(identity));

        completed

    }
}

fn main() {
    simple_logger::init_with_level(log::Level::Info).unwrap();

    lambda!(handler);
}
