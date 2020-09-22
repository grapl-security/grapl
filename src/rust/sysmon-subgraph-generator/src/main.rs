#![type_length_limit = "1334469"]

mod metrics;

use std::collections::HashSet;
use std::fmt::Debug;
use std::io::Cursor;
use std::marker::PhantomData;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use aws_lambda_events::event::s3::{
    S3Bucket, S3Entity, S3Event, S3EventRecord, S3Object, S3RequestParameters, S3UserIdentity,
};
use aws_lambda_events::event::sqs::SqsEvent;
use chrono::prelude::*;
use failure::bail;
use failure::Error;
use lambda_runtime::error::HandlerError;
use lambda_runtime::lambda;
use lambda_runtime::Context;
use rusoto_core::credential::AwsCredentials;
use rusoto_core::{HttpClient, Region, RusotoError};
use rusoto_s3::GetObjectRequest;
use rusoto_s3::{S3Client, S3};
use rusoto_sqs::GetQueueUrlRequest;
use rusoto_sqs::{ListQueuesRequest, SendMessageRequest, Sqs, SqsClient};
use serde::Deserialize;
use sqs_lambda::cache::Cache;
use sqs_lambda::cache::{CacheResponse, NopCache};
use sqs_lambda::completion_event_serializer::CompletionEventSerializer;
use sqs_lambda::event_decoder::PayloadDecoder;
use sqs_lambda::event_handler::{Completion, EventHandler, OutputEvent};
use sqs_lambda::event_processor::{EventProcessor, EventProcessorActor};
use sqs_lambda::event_retriever::S3PayloadRetriever;
use sqs_lambda::local_sqs_service::local_sqs_service;
use sqs_lambda::redis_cache::RedisCache;
use sqs_lambda::s3_event_emitter::S3EventEmitter;
use sqs_lambda::sqs_completion_handler::{
    CompletionPolicy, SqsCompletionHandler, SqsCompletionHandlerActor,
};
use sqs_lambda::sqs_consumer::{ConsumePolicy, SqsConsumer, SqsConsumerActor};
use sqs_lambda::sqs_service::sqs_service;
use sysmon::*;
use tokio::runtime::Runtime;
use uuid::Uuid;

use async_trait::async_trait;
use graph_descriptions::file::FileState;
use graph_descriptions::graph_description::*;
use graph_descriptions::network_connection::NetworkConnectionState;
use graph_descriptions::process::ProcessState;
use graph_descriptions::process_inbound_connection::ProcessInboundConnectionState;
use graph_descriptions::process_outbound_connection::ProcessOutboundConnectionState;
use lazy_static::lazy_static;

use graph_generator_lib::*;

use graph_descriptions::node::NodeT;
use log::*;

use crate::metrics::SysmonSubgraphGeneratorMetrics;
use grapl_config::*;

macro_rules! log_time {
    ($msg:expr, $x:expr) => {{
        let mut sw = stopwatch::Stopwatch::start_new();
        #[allow(path_statements)]
        let result = $x;
        sw.stop();
        info!("{} {} milliseconds", $msg, sw.elapsed_ms());
        result
    }};
}

/// Returns the provided file path with the Windows Zone Identifier removed if present.
///
/// When files are downloaded via a browser (e.g. Internet Explorer), an alternative data stream (ADS) may be created
/// to store additional meta-data about the file. These ADS files are suffixed with `:Zone.Identifier`.
///
/// The purpose of these ADS files is to provide context to users and Windows Policy controls to keep users safe.
///
/// For example, Microsoft Word may open potentially unsafe `.docx` files marked with `:Zone.Identifier`
/// in Protected View if the URL security zone is not well trusted.
///
/// ["What are Zone Identifier Files?"](https://stackoverflow.com/questions/4496697/what-are-zone-identifier-files)
fn strip_file_zone_identifier(path: &str) -> &str {
    if path.ends_with(":Zone.Identifier") {
        &path[0..path.len() - ":Zone.Identifier".len()]
    } else {
        path
    }
}

/// Gets the name of the process given a path to the executable.
fn get_image_name(image_path: &str) -> Option<String> {
    image_path
        .split('\\')
        .last()
        .map(|name| name.replace("- ", "").replace('\\', ""))
}

/// Converts a Sysmon UTC string to UNIX Epoch time
///
/// If the provided string is not parseable as a UTC timestamp, an error is returned.
pub fn utc_to_epoch(utc: &str) -> Result<u64, Error> {
    let dt = NaiveDateTime::parse_from_str(utc, "%Y-%m-%d %H:%M:%S%.3f")?;

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

/// Creates a subgraph describing a `ProcessCreateEvent`.
///
/// Subgraph generation for a `ProcessCreateEvent` includes the following:
/// * An `Asset` node - indicating the asset in which the process was created
/// * A parent `Process` node - indicating the process that created the subject process
/// * A subject `Process` node - indicating the process created per the `ProcessCreateEvent`
/// * A process `File` node - indicating the file executed in creating the new process
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

    graph.add_edge(
        "asset_processes",
        asset.clone_node_key(),
        child.clone_node_key(),
    );

    graph.add_edge(
        "process_asset",
        child.clone_node_key(),
        asset.clone_node_key(),
    );

    graph.add_edge(
        "bin_file",
        child.clone_node_key(),
        child_exe.clone_node_key(),
    );

    graph.add_edge("children", parent.clone_node_key(), child.clone_node_key());
    graph.add_edge("parent", child.clone_node_key(), parent.clone_node_key());

    graph.add_node(asset);
    graph.add_node(parent);
    graph.add_node(child);
    graph.add_node(child_exe);

    Ok(graph)
}

/// Creates a subgraph describing an outbound `NetworkEvent`
///
/// Subgraph generation for an outbound `NetworkEvent` includes the following:
/// * An `Asset` node - indicating the asset in which the outbound `NetworkEvent` occurred
/// * A `Process` node - indicating the process which triggered the outbound `NetworkEvent`
/// * A subject `OutboundConnection` node - indicating the network connection triggered by the process
/// * Source and Destination IP Address and Port nodes
/// * IP connection and Network connection nodes
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
        .asset_id(conn_log.system.computer.computer.clone())
        .hostname(conn_log.system.computer.computer.clone())
        .state(ProcessState::Existing)
        .process_id(conn_log.event_data.process_id)
        .last_seen_timestamp(timestamp)
        .build()
        .expect("outbound_connection.process");

    let outbound = ProcessOutboundConnectionBuilder::default()
        .asset_id(conn_log.system.computer.computer.clone())
        .hostname(conn_log.system.computer.computer.clone())
        .state(ProcessOutboundConnectionState::Connected)
        .ip_address(conn_log.event_data.source_ip.clone())
        .protocol(conn_log.event_data.protocol.clone())
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
    graph.add_edge("asset_ip", asset.clone_node_key(), src_ip.clone_node_key());

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
/// Creates a subgraph describing an inbound `NetworkEvent`
///
/// The subgraph generated is similar to the graph generated by [handle_outbound_connection]
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
        .asset_id(conn_log.system.computer.computer.clone())
        .hostname(conn_log.system.computer.computer.clone())
        .state(ProcessState::Existing)
        .process_id(conn_log.event_data.process_id)
        .last_seen_timestamp(timestamp)
        .build()
        .expect("inbound_connection.process");

    let outbound = ProcessInboundConnectionBuilder::default()
        .asset_id(conn_log.system.computer.computer.clone())
        .hostname(conn_log.system.computer.computer.clone())
        .state(ProcessInboundConnectionState::Bound)
        .port(conn_log.event_data.source_port)
        .ip_address(conn_log.event_data.source_ip.clone())
        .protocol(conn_log.event_data.protocol.clone())
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
    graph.add_edge("asset_ip", asset.clone_node_key(), src_ip.clone_node_key());

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

/// Creates a subgrqph describing a `FileCreateEvent`
///
/// The subgraph generation for a `FileCreateEvent` includes the following:
/// * A creator `Process` node - denotes the process that created the file
/// * A subject `File` node - the file that is created as part of this event
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
        .file_path(strip_file_zone_identifier(
            &file_create.event_data.target_filename,
        ))
        .created_timestamp(timestamp)
        .build()
        .unwrap();

    graph.add_edge(
        "created_files",
        creator.clone_node_key(),
        file.clone_node_key(),
    );
    graph.add_node(creator);
    graph.add_node(file);

    Ok(graph)
}

/// A [PayloadDecoder] used to decompress zstd encoded events sent to an [EventHandler].
///
/// This `struct` is typically used in conjunction with a subsequent call to [run_graph_generator].
#[derive(Debug, Clone, Default)]
pub struct ZstdDecoder;

impl PayloadDecoder<Vec<u8>> for ZstdDecoder {
    fn decode(&mut self, body: Vec<u8>) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut decompressed = Vec::new();
        let mut body = Cursor::new(&body);

        zstd::stream::copy_decode(&mut body, &mut decompressed)?;

        Ok(decompressed)
    }
}

#[derive(Clone)]
struct SysmonSubgraphGenerator<C>
where
    C: Cache + Clone + Send + Sync + 'static,
{
    cache: C,
    metrics: SysmonSubgraphGeneratorMetrics,
}

impl<C> SysmonSubgraphGenerator<C>
where
    C: Cache + Clone + Send + Sync + 'static,
{
    pub fn new(cache: C, metrics: SysmonSubgraphGeneratorMetrics) -> Self {
        Self { cache, metrics }
    }
}

#[async_trait]
impl<C> EventHandler for SysmonSubgraphGenerator<C>
where
    C: Cache + Clone + Send + Sync + 'static,
{
    type InputEvent = Vec<u8>;
    type OutputEvent = Graph;
    type Error = sqs_lambda::error::Error;

    async fn handle_event(
        &mut self,
        events: Vec<u8>,
    ) -> OutputEvent<Self::OutputEvent, Self::Error> {
        info!("Handling raw event");

        let mut failed: Option<failure::Error> = None;

        let events: Vec<_> = log_time!(
            "event split",
            events
                .split(|i| &[*i][..] == &b"\n"[..])
                .map(String::from_utf8_lossy)
                .filter(|event| {
                    (!event.is_empty() && event != "\n")
                        && (event.contains(&"EventID>1<"[..])
                            || event.contains(&"EventID>3<"[..])
                            || event.contains(&"EventID>11<"[..]))
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
                        })()
                        .unwrap_err(),
                    );
                    continue;
                }
            };

            match self.cache.get(event.clone()).await {
                Ok(CacheResponse::Hit) => {
                    info!("Got cached response");
                    continue;
                }
                Err(e) => warn!("Cache failed with: {:?}", e),
                _ => (),
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
                // Event::InboundNetwork(event) => {
                //     match handle_inbound_connection(event) {
                //         Ok(event) => Some(event),
                //         Err(e) => {
                //             warn!("Failed to process inbound network event: {}", e);
                //             None
                //         }
                //     }
                // }
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
        self.metrics.report_handle_event_success(&failed);

        let mut completed = if let Some(ref e) = failed {
            OutputEvent::new(Completion::Partial((
                final_subgraph,
                sqs_lambda::error::Error::ProcessingError(e.to_string()),
            )))
        } else {
            OutputEvent::new(Completion::Total(final_subgraph))
        };

        identities
            .into_iter()
            .for_each(|identity| completed.add_identity(identity));

        completed
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    grapl_config::init_grapl_env!();
    info!("Starting sysmon-subgraph-generator");

    let metrics = SysmonSubgraphGeneratorMetrics::new();

    if grapl_config::is_local() {
        let generator = SysmonSubgraphGenerator::new(NopCache {}, metrics);

        run_graph_generator(generator, ZstdDecoder::default()).await;
    } else {
        let generator = SysmonSubgraphGenerator::new(event_cache().await, metrics);

        run_graph_generator(generator, ZstdDecoder::default()).await;
    }

    Ok(())
}
