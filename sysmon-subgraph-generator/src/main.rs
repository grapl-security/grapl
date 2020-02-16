extern crate aws_lambda_events;
extern crate chrono;
extern crate failure;
extern crate futures;
extern crate graph_descriptions;

extern crate lambda_runtime as lambda;
#[macro_use]
extern crate lazy_static;
extern crate log;
extern crate rayon;
extern crate regex;
extern crate rusoto_core;
extern crate rusoto_s3;
extern crate rusoto_sqs;
extern crate serde;
extern crate simple_logger;
extern crate sqs_lambda;
extern crate stopwatch;
extern crate sysmon;
extern crate uuid;

use std::borrow::Cow;
use std::marker::PhantomData;
use std::str::FromStr;
use std::sync::Arc;

use aws_lambda_events::event::sqs::{SqsEvent, SqsMessage};
use chrono::prelude::*;
use failure::bail;
use failure::Error;
use futures::{Future, Stream};
use lambda::Context;
use lambda::error::HandlerError;
use lambda::Handler;
use lambda::lambda;
use log::*;
use log::error;
use rayon::iter::Either;
use rayon::prelude::*;
use regex::Regex;
use rusoto_core::Region;
use rusoto_s3::{GetObjectRequest, S3};
use rusoto_s3::S3Client;
use rusoto_sqs::{GetQueueUrlRequest, Sqs, SqsClient};
use serde::Deserialize;

use sqs_lambda::completion_event_serializer::CompletionEventSerializer;
use sqs_lambda::event_decoder::PayloadDecoder;
use sqs_lambda::event_emitter::S3EventEmitter;
use sqs_lambda::event_handler::{Completion, EventHandler, OutputEvent};
use sqs_lambda::event_processor::{EventProcessor, EventProcessorActor};
use sqs_lambda::event_retriever::S3PayloadRetriever;
use sqs_lambda::redis_cache::RedisCache;
use sqs_lambda::sqs_completion_handler::{CompletionPolicy, SqsCompletionHandler, SqsCompletionHandlerActor};
use sqs_lambda::sqs_consumer::{ConsumePolicy, SqsConsumer, SqsConsumerActor};
use sqs_lambda::cache::CacheResponse;
use async_trait::async_trait;


use sysmon::*;
use uuid::Uuid;

use graph_descriptions::*;
use graph_descriptions::file::FileState;
use graph_descriptions::graph_description::*;
use graph_descriptions::network_connection::NetworkConnectionState;

use graph_descriptions::process::ProcessState;
use graph_descriptions::process_inbound_connection::ProcessInboundConnectionState;
use graph_descriptions::process_outbound_connection::ProcessOutboundConnectionState;
use crate::graph_descriptions::node::NodeT;

use std::io::Cursor;
use sqs_lambda::cache::Cache;
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use std::collections::HashSet;


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

fn strip_file_zone_identifier(path: &str) -> &str {
    if path.ends_with(":Zone.Identifier") {
        &path[0..path.len() - ":Zone.Identifier".len()]
    } else {
        path
    }
}

fn is_internal_ip(ip: &str) -> bool {
    lazy_static!(
        static ref RE: Regex = Regex::new(
            r"/(^127\.)|(^192\.168\.)|(^10\.)|(^172\.1[6-9]\.)|(^172\.2[0-9]\.)|(^172\.3[0-1]\.)|(^::1$)|(^[fF][cCdD])/"
        ).expect("is_internal_ip regex");
    );

    RE.is_match(ip)
}

fn get_image_name(image_path: &str) -> Option<String> {
    image_path.split('\\').last().map(|name| {
        name.replace("- ", "").replace('\\', "")
    })
}

pub fn utc_to_epoch(utc: &str) -> Result<u64, Error> {
    let dt = NaiveDateTime::parse_from_str(
        utc, "%Y-%m-%d %H:%M:%S%.3f")?;

    let dt: DateTime<Utc> = DateTime::from_utc(dt, Utc);
    let ts = dt.timestamp_millis();

    if ts < 0 {
        bail!("Timestamp is negative")
    }


    if ts < 1_000_000_000_000 {
        Ok(ts as u64 * 1000)
    } else {
        Ok(ts as u64)
    }
}

fn handle_process_start(process_start: &ProcessCreateEvent) -> Result<Graph, Error> {
    let timestamp = utc_to_epoch(&process_start.event_data.utc_time)?;
    let mut graph = Graph::new(timestamp);

    let asset = AssetBuilder::default()
        .asset_id(process_start.system.computer.computer.clone())
        .hostname(process_start.system.computer.computer.clone())
        .build()
        .unwrap();

    let parent = ProcessBuilder::default()
        .asset_id(process_start.system.computer.computer.clone())
        .state(ProcessState::Existing)
        .process_id(process_start.event_data.parent_process_id)
        .process_name(get_image_name(&process_start.event_data.parent_image.clone()).unwrap())
        .process_command_line(&process_start.event_data.parent_command_line.command_line)
        .last_seen_timestamp(timestamp)
//        .created_timestamp(process_start.event_data.parent_process_guid.get_creation_timestamp())
        .build()
        .expect("process_start.parent");

    let child = ProcessBuilder::default()
        .asset_id(process_start.system.computer.computer.clone())
        .process_name(get_image_name(&process_start.event_data.image.clone()).unwrap())
        .process_command_line(&process_start.event_data.command_line.command_line)
        .state(ProcessState::Created)
        .process_id(process_start.event_data.process_id)
        .created_timestamp(timestamp)
        .build()
        .expect("process_start.child");

    let child_exe = FileBuilder::default()
        .asset_id(process_start.system.computer.computer.clone())
        .state(FileState::Existing)
        .last_seen_timestamp(timestamp)
        .file_path(strip_file_zone_identifier(&process_start.event_data.image))
        .build()
        .expect("process_start.child_Exe");

    graph.add_edge("asset_processes",
                   asset.clone_node_key(),
                   child.clone_node_key(),
    );

    graph.add_edge("bin_file",
                   child.clone_node_key(),
                   child_exe.clone_node_key(),
    );

    graph.add_edge("children",
                   parent.clone_node_key(),
                   child.clone_node_key());

    graph.add_node(asset);
    graph.add_node(parent);
    graph.add_node(child);
    graph.add_node(child_exe);

    Ok(graph)
}

fn handle_outbound_connection(conn_log: &NetworkEvent) -> Result<Graph, Error> {
    let timestamp = utc_to_epoch(&conn_log.event_data.utc_time)?;

    let mut graph = Graph::new(timestamp);

    let asset = AssetBuilder::default()
        .asset_id(conn_log.system.computer.computer.clone())
        .hostname(conn_log.system.computer.computer.clone())
        .build()
        .expect("outbound_connection.asset");

    // A process creates an outbound connection to dst_port
    let process = ProcessBuilder::default()
        .hostname(conn_log.system.computer.computer.clone())
        .state(ProcessState::Existing)
        .process_id(conn_log.event_data.process_id)
        .last_seen_timestamp(timestamp)
        .build()
        .expect("outbound_connection.process");

    let outbound = ProcessOutboundConnectionBuilder::default()
        .hostname(conn_log.system.computer.computer.clone())
        .state(ProcessOutboundConnectionState::Connected)
        .port(conn_log.event_data.source_port)
        .created_timestamp(timestamp)
        .build()
        .expect("outbound_connection.outbound");

    let src_ip = IpAddressBuilder::default()
        .ip_address(conn_log.event_data.source_ip.clone())
        .last_seen_timestamp(timestamp)
        .build()
        .expect("outbound_connection.src_ip");

    let dst_ip = IpAddressBuilder::default()
        .ip_address(conn_log.event_data.destination_ip.clone())
        .last_seen_timestamp(timestamp)
        .build()
        .expect("outbound_connection.dst_ip");

    let src_port = IpPortBuilder::default()
        .ip_address(conn_log.event_data.source_ip.clone())
        .port(conn_log.event_data.source_port)
        .protocol(conn_log.event_data.protocol.clone())
        .build()
        .expect("outbound_connection.src_port");

    let dst_port = IpPortBuilder::default()
        .ip_address(conn_log.event_data.destination_ip.clone())
        .port(conn_log.event_data.destination_port)
        .protocol(conn_log.event_data.protocol.clone())
        .build()
        .unwrap();

    let network_connection = NetworkConnectionBuilder::default()
        .state(NetworkConnectionState::Created)
        .src_ip_address(conn_log.event_data.source_ip.clone())
        .src_port(conn_log.event_data.source_port)
        .dst_ip_address(conn_log.event_data.destination_ip.clone())
        .dst_port(conn_log.event_data.destination_port)
        .protocol(conn_log.event_data.protocol.clone())
        .created_timestamp(timestamp)
        .build()
        .expect("outbound_connection.network_connection");

    let ip_connection = IpConnectionBuilder::default()
        .state(NetworkConnectionState::Created)
        .src_ip_address(conn_log.event_data.source_ip.clone())
        .dst_ip_address(conn_log.event_data.destination_ip.clone())
        .protocol(conn_log.event_data.protocol.clone())
        .created_timestamp(timestamp)
        .build()
        .expect("outbound_connection.ip_connection");

    // An asset is assigned an IP
    graph.add_edge(
        "asset_ip",
        asset.clone_node_key(),
        src_ip.clone_node_key(),
    );

    // A process spawns on an asset
    graph.add_edge(
        "asset_processes",
        asset.clone_node_key(),
        process.clone_node_key(),
    );

    // A process creates a connection
    graph.add_edge(
        "created_connection",
        process.clone_node_key(),
        outbound.clone_node_key(),
    );

    // The connection is over an IP + Port
    graph.add_edge(
        "connected_over",
        outbound.clone_node_key(),
        src_port.clone_node_key(),
    );

    // The outbound process connection is to a dst ip + port
    graph.add_edge(
        "external_connection",
        outbound.clone_node_key(),
        dst_port.clone_node_key(),
    );

    // There is a network connection between the src and dst ports
    graph.add_edge(
        "outbound_connection_to",
        src_port.clone_node_key(),
        network_connection.clone_node_key(),
    );

    graph.add_edge(
        "inbound_connection_to",
        network_connection.clone_node_key(),
        dst_port.clone_node_key(),
    );

    // There is also a connection between the two IP addresses

    graph.add_edge(
        "ip_connection_to",
        src_ip.clone_node_key(),
        ip_connection.clone_node_key(),
    );

    graph.add_edge(
        "ip_connection_to",
        ip_connection.clone_node_key(),
        dst_ip.clone_node_key(),
    );

    graph.add_node(asset);
    graph.add_node(process);
    graph.add_node(outbound);
    graph.add_node(src_ip);
    graph.add_node(dst_ip);
    graph.add_node(src_port);
    graph.add_node(dst_port);
    graph.add_node(network_connection);
    graph.add_node(ip_connection);

    Ok(graph)
}

// Inbound is the 'src' in sysmon
fn handle_inbound_connection(conn_log: &NetworkEvent) -> Result<Graph, Error> {
    let timestamp = utc_to_epoch(&conn_log.event_data.utc_time)?;

    let mut graph = Graph::new(timestamp);

    let asset = AssetBuilder::default()
        .asset_id(conn_log.system.computer.computer.clone())
        .hostname(conn_log.system.computer.computer.clone())
        .build()
        .expect("inbound_connection.asset");

    // A process creates an outbound connection to dst_port
    let process = ProcessBuilder::default()
        .hostname(conn_log.system.computer.computer.clone())
        .state(ProcessState::Existing)
        .process_id(conn_log.event_data.process_id)
        .last_seen_timestamp(timestamp)
        .build()
        .expect("inbound_connection.process");

    let outbound = ProcessInboundConnectionBuilder::default()
        .hostname(conn_log.system.computer.computer.clone())
        .state(ProcessInboundConnectionState::Bound)
        .port(conn_log.event_data.source_port)
        .created_timestamp(timestamp)
        .build()
        .expect("inbound_connection.outbound");

    let src_ip = IpAddressBuilder::default()
        .ip_address(conn_log.event_data.source_ip.clone())
        .last_seen_timestamp(timestamp)
        .build()
        .expect("inbound_connection.src_ip");

    let dst_ip = IpAddressBuilder::default()
        .ip_address(conn_log.event_data.destination_ip.clone())
        .last_seen_timestamp(timestamp)
        .build()
        .expect("inbound_connection.dst_ip");

    let src_port = IpPortBuilder::default()
        .ip_address(conn_log.event_data.source_ip.clone())
        .port(conn_log.event_data.source_port)
        .protocol(conn_log.event_data.protocol.clone())
        .build()
        .expect("inbound_connection.src_port");

    let dst_port = IpPortBuilder::default()
        .ip_address(conn_log.event_data.destination_ip.clone())
        .port(conn_log.event_data.destination_port)
        .protocol(conn_log.event_data.protocol.clone())
        .build()
        .expect("inbound_connection.dst_port");

    let network_connection = NetworkConnectionBuilder::default()
        .state(NetworkConnectionState::Created)
        .src_ip_address(conn_log.event_data.source_ip.clone())
        .src_port(conn_log.event_data.source_port)
        .dst_ip_address(conn_log.event_data.destination_ip.clone())
        .dst_port(conn_log.event_data.destination_port)
        .created_timestamp(timestamp)
        .build()
        .expect("inbound_connection.network_connection");

    // An asset is assigned an IP
    graph.add_edge(
        "asset_ip",
        asset.clone_node_key(),
        src_ip.clone_node_key(),
    );

    // A process spawns on an asset
    graph.add_edge(
        "asset_processes",
        asset.clone_node_key(),
        process.clone_node_key(),
    );

    // A process creates a connection
    graph.add_edge(
        "created_connections",
        process.clone_node_key(),
        outbound.clone_node_key(),
    );

    // The connection is over an IP + Port
    graph.add_edge(
        "connected_over",
        outbound.clone_node_key(),
        src_port.clone_node_key(),
    );

    // The connection is to a dst ip + port
    graph.add_edge(
        "connected_to",
        outbound.clone_node_key(),
        dst_port.clone_node_key(),
    );

    // There is a network connection between the src and dst ports
    graph.add_edge(
        "outbound_connection_to",
        src_port.clone_node_key(),
        network_connection.clone_node_key(),
    );

    graph.add_edge(
        "inbound_connection_to",
        network_connection.clone_node_key(),
        dst_port.clone_node_key(),
    );

    graph.add_node(asset);
    graph.add_node(process);
    graph.add_node(outbound);
    graph.add_node(src_ip);
    graph.add_node(dst_ip);
    graph.add_node(src_port);
    graph.add_node(dst_port);
    graph.add_node(network_connection);

    Ok(graph)
}

fn handle_file_create(file_create: &FileCreateEvent) -> Result<Graph, Error> {
    let timestamp = utc_to_epoch(&file_create.event_data.creation_utc_time)?;
    let mut graph = Graph::new(timestamp);

    let creator = ProcessBuilder::default()
        .asset_id(file_create.system.computer.computer.clone())
        .state(ProcessState::Existing)
        .process_id(file_create.event_data.process_id)
        .process_name(get_image_name(&file_create.event_data.image.clone()).unwrap())
        .last_seen_timestamp(timestamp)
//        .created_timestamp(file_create.event_data.process_guid.get_creation_timestamp())
        .build()
        .unwrap();

    let file = FileBuilder::default()
        .asset_id(file_create.system.computer.computer.clone())
        .state(FileState::Created)
        .file_path(strip_file_zone_identifier(&file_create.event_data.target_filename))
        .created_timestamp(timestamp)
        .build()
        .unwrap();


    graph.add_edge("created_files",
                   creator.clone_node_key(),
                   file.clone_node_key());
    graph.add_node(creator);
    graph.add_node(file);

    Ok(graph)
}

#[derive(Clone, Debug, Default)]
pub struct SubgraphSerializer {
    proto: Vec<u8>,
}

impl CompletionEventSerializer for SubgraphSerializer {
    type CompletedEvent = Graph;
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

        for sg in completed_events.iter() {
            pre_nodes += sg.nodes.len();
            pre_edges += sg.edges.len();
            subgraph.merge(sg);
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

#[derive(Debug, Clone, Default)]
pub struct ZstdDecoder;

impl PayloadDecoder<Vec<u8>> for ZstdDecoder
{
    fn decode(&mut self, body: Vec<u8>) -> Result<Vec<u8>, Box<dyn std::error::Error>>
    {
        let mut decompressed = Vec::new();

        let mut body = Cursor::new(&body);

        zstd::stream::copy_decode(&mut body, &mut decompressed)?;

        Ok(decompressed)
    }
}


#[derive(Clone)]
struct SysmonSubgraphGenerator {
    cache: RedisCache,
}

impl SysmonSubgraphGenerator {
    pub fn new(cache: RedisCache) -> Self {
        Self { cache }
    }
}

#[async_trait]
impl EventHandler for SysmonSubgraphGenerator
{
    type InputEvent = Vec<u8>;
    type OutputEvent = Graph;
    type Error = Arc<failure::Error>;

    async fn handle_event(&mut self, events: Vec<u8>) -> OutputEvent<Self::OutputEvent, Self::Error> {
        info!("Handling raw event");

        let mut failed: Option<failure::Error> = None;

        let events: Vec<_> = log_time!(
            "event split",
            events.split(|i| &[*i][..] == &b"\n"[..])
            .map(String::from_utf8_lossy)
            .filter(|event| {
                (!event.is_empty() && event != "\n") &&
                (event.contains(&"EventID>1<"[..]) || event.contains(&"EventID>3<"[..])  || event.contains(&"EventID>11<"[..]))
            })
            .collect()
        );

        info!("Handling {} events", events.len());

        let mut identities = Vec::with_capacity(events.len());

        let mut final_subgraph = Graph::new(0);

        for event in events {
            let des_event = Event::from_str(&event);
            let event = match des_event {
                Ok(event) => event,
                Err(e) => {
                    warn!("Failed to deserialize event: {}, {}", e, event);
                    failed = Some(
                        (|| {
                            bail!("Failed: {}", e);
                            Ok(())
                        })().unwrap_err()
                    );
                    continue;
                }
            };

            match self.cache.get(event.clone()).await {
                Ok(CacheResponse::Hit) =>  {
                    info!("Got cached response");
                    continue
                },
                Err(e) => warn!("Cache failed with: {:?}", e),
                _ => ()
            };

            let graph = match event.clone() {
                Event::ProcessCreate(event) => {
                    info!("Handling process create");

                    match handle_process_start(&event) {
                        Ok(event) => event,
                        Err(e) => {
                            warn!("Failed to process process start event: {}", e);
                            failed = Some(e);
                            continue;
                        }
                    }
                }
                Event::FileCreate(event) => {
                    info!("FileCreate");

                    match handle_file_create(&event) {
                        Ok(event) => event,
                        Err(e) => {
                            warn!("Failed to process file create event: {}", e);
                            failed = Some(e);
                            continue;
                        }
                    }
                }
//                    Event::InboundNetwork(event) => {
//                        match handle_inbound_connection(event) {
//                            Ok(event) => Some(event),
//                            Err(e) => {
//                                warn!("Failed to process inbound network event: {}", e);
//                                None
//                            }
//                        }
//                    }
                Event::OutboundNetwork(event) => {
                    info!("OutboundNetwork");
                    match handle_outbound_connection(&event) {
                        Ok(event) => event,
                        Err(e) => {
                            warn!("Failed to process outbound network event: {}", e);
                            failed = Some(e);
                            continue;
                        }
                    }
                }
                catch => {
                    warn!("Unsupported event_type: {:?}", catch);
                    continue;
                }
            };
            identities.push(event);

            final_subgraph.merge(&graph);
        }

        info!("Completed mapping {} subgraphs", identities.len());

        let mut completed = if let Some(e) = failed {
            OutputEvent::new(
                Completion::Partial(
                    (
                        final_subgraph,
                        Arc::new(e)
                    )
                )
            )
        } else {
            OutputEvent::new(Completion::Total(final_subgraph))
        };

        identities.into_iter().for_each(|identity| completed.add_identity(identity));

        completed
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
                    let sysmon_event_cache_addr = std::env::var("SYSMON_EVENT_CACHE_ADDR").expect("SYSMON_EVENT_CACHE_ADDR");
                    let sysmon_event_cache_port = std::env::var("SYSMON_EVENT_CACHE_PORT").expect("SYSMON_EVENT_CACHE_PORT");

                    format!(
                        "{}:{}",
                        sysmon_event_cache_addr,
                        sysmon_event_cache_port,
                    )
                };

                info!("Redis cache: {}", &cache_address);

                let bucket = bucket_prefix + "-unid-subgraphs-generated-bucket";
                info!("Output events to: {}", bucket);
                let region = {
                    let region_str = std::env::var("AWS_REGION").expect("AWS_REGION");
                    Region::from_str(&region_str).expect("Region error")
                };

                let cache = RedisCache::new(cache_address.to_owned()).await.expect("Could not create redis client");

                let node_identifier = SysmonSubgraphGenerator::new(
                    cache.clone()
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
                        cache.clone()
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
                            S3PayloadRetriever::new(S3Client::new(region.clone()), ZstdDecoder::default()),
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


fn main() -> Result<(), Box<dyn std::error::Error>> {
    simple_logger::init_with_level(log::Level::Info).unwrap();

    info!("Starting lambda");
    lambda!(handler);
    Ok(())
}


#[cfg(test)]
mod tests {
    use std::time::Duration;

    use futures::executor::block_on;

    use rusoto_core::credential::StaticProvider;
    use rusoto_core::HttpClient;
    use rusoto_s3::CreateBucketRequest;

    use super::*;

    #[test]
    fn parse_time() {
        let utc_time = "2017-04-28 22:08:22.025";
        let ts = utc_to_epoch(utc_time).expect("parsing utc_time failed");
        println!("{}", ts);
    }

    #[test]
    fn test_handler() {
        let region = Region::Custom {
            name: "us-east-1".to_string(),
            endpoint: "http://127.0.0.1:9000".to_string(),
        };

        std::env::set_var("BUCKET_PREFIX", "unique_id");

        /* let handler = SysmonSubgraphGenerator::new(
            move |generated_subgraphs| {
                println!("generated subgraphs");
                Ok(())
            }
        );

        block_on(handler.handle_event(vec![]))
            .expect("handle_event failed"); */
    }
}

