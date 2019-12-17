extern crate aws_lambda_events;
#[macro_use]
extern crate failure;
extern crate graph_descriptions;
extern crate graph_generator_lib;
extern crate lambda_runtime as lambda;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate regex;
extern crate rusoto_core;
extern crate rusoto_s3;
extern crate rusoto_sqs;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate simple_logger;
extern crate sqs_lambda;
extern crate uuid;

use std::marker::PhantomData;
use std::str::FromStr;
use std::sync::Arc;

use aws_lambda_events::event::sqs::{SqsEvent, SqsMessage};
use failure::Error;
use lambda::Context;
use lambda::error::HandlerError;
use lambda::Handler;
use lambda::lambda;
use regex::Regex;
use rusoto_core::Region;
use rusoto_s3::{GetObjectRequest, S3};
use rusoto_s3::S3Client;
use rusoto_sqs::{GetQueueUrlRequest, Sqs, SqsClient};
use serde_json::Value;
use sqs_lambda::BlockingSqsCompletionHandler;
use sqs_lambda::EventHandler;
use sqs_lambda::events_from_s3_sns_sqs;
use sqs_lambda::S3EventRetriever;
use sqs_lambda::SqsService;
use sqs_lambda::ZstdJsonDecoder;

use graph_descriptions::*;
use graph_descriptions::file::FileState;
use graph_descriptions::graph_description::*;
use graph_descriptions::network_connection::NetworkConnectionState;
use graph_descriptions::process::ProcessState;
use graph_descriptions::process_inbound_connection::ProcessInboundConnectionState;
use graph_descriptions::process_outbound_connection::ProcessOutboundConnectionState;
use graph_generator_lib::upload_subgraphs;

use crate::graph_descriptions::node::NodeT;

#[derive(Serialize, Deserialize)]
pub struct ProcessStart {
    process_id: u64,
    parent_process_id: u64,
    name: String,
    hostname: String,
    arguments: String,
    timestamp: u64,
    exe: Option<String>,
    sourcetype: String,
}

#[derive(Serialize, Deserialize)]
pub struct ProcessStop {
    process_id: u64,
    name: String,
    hostname: String,
    timestamp: u64,
    sourcetype: String,
}

#[derive(Serialize, Deserialize)]
pub struct FileCreate {
    creator_process_id: u64,
    creator_process_name: Option<String>,
    path: String,
    hostname: String,
    timestamp: u64,
    sourcetype: String,
}

#[derive(Serialize, Deserialize)]
pub struct FileDelete {
    deleter_process_id: u64,
    deleter_process_name: Option<String>,
    path: String,
    hostname: String,
    timestamp: u64,
    sourcetype: String,
}

#[derive(Serialize, Deserialize)]
pub struct FileRead {
    reader_process_id: u64,
    reader_process_name: Option<String>,
    path: String,
    hostname: String,
    timestamp: u64,
    sourcetype: String,
}

#[derive(Serialize, Deserialize)]
pub struct FileWrite {
    writer_pid: u64,
    writer_process_name: Option<String>,
    path: String,
    hostname: String,
    timestamp: u64,
    sourcetype: String,
}

#[derive(Serialize, Deserialize)]
pub struct ProcessOutboundConnectionLog {
    pid: u64,
    protocol: String,
    src_port: u32,
    dst_port: u32,
    src_hostname: String,
    src_ip_addr: String,
    dst_ip_addr: String,
    timestamp: u64,
    sourcetype: String,
}

// In an inbound connection "src" is where the connection is coming from
#[derive(Serialize, Deserialize)]
pub struct ProcessInboundConnectionLog {
    /// The pid of the process receiving the connection
    pid: u64,
    src_ip_addr: String,
    src_port: u32,
    dst_port: u32,
    dst_hostname: String,
    dst_ip_addr: String,
    protocol: String,
    timestamp: u64,
    sourcetype: String,
}

#[derive(Serialize, Deserialize)]
pub struct ProcessPortBindLog {
    pid: u64,
    bound_port: u64,
    hostname: String,
    timestamp: u64,
    sourcetype: String,
}

fn is_internal_ip(ip: &str) -> bool {
    lazy_static!(
        static ref RE: Regex = Regex::new(
            r"/(^127\.)|(^192\.168\.)|(^10\.)|(^172\.1[6-9]\.)|(^172\.2[0-9]\.)|(^172\.3[0-1]\.)|(^::1$)|(^[fF][cCdD])/"
        ).expect("is_internal_ip regex");
    );

    RE.is_match(ip)
}

fn handle_outbound_traffic(conn_log: ProcessOutboundConnectionLog) -> Graph {
    let mut graph = Graph::new(conn_log.timestamp);

    let asset = AssetBuilder::default()
        .hostname(conn_log.src_hostname.clone())
        .build()
        .unwrap();

    // A process creates an outbound connection to dst_port
    let process = ProcessBuilder::default()
        .asset_id(conn_log.src_hostname.clone())
        .state(ProcessState::Existing)
        .process_id(conn_log.pid)
        .last_seen_timestamp(conn_log.timestamp)
        .build()
        .unwrap();

    let outbound = ProcessOutboundConnectionBuilder::default()
        .asset_id(conn_log.src_hostname.clone())
        .state(ProcessOutboundConnectionState::Connected)
        .port(conn_log.src_port)
        .created_timestamp(conn_log.timestamp)
        .build()
        .unwrap();

    let src_ip = IpAddressBuilder::default()
        .ip_address(conn_log.src_ip_addr.clone())
        .last_seen_timestamp(conn_log.timestamp)
        .build()
        .unwrap();

    let dst_ip = IpAddressBuilder::default()
        .ip_address(conn_log.dst_ip_addr.clone())
        .last_seen_timestamp(conn_log.timestamp)
        .build()
        .unwrap();

    let src_port = IpPortBuilder::default()
        .ip_address(conn_log.src_ip_addr.clone())
        .port(conn_log.src_port)
        .protocol(conn_log.protocol.clone())
        .build()
        .unwrap();

    let dst_port = IpPortBuilder::default()
        .ip_address(conn_log.dst_ip_addr.clone())
        .port(conn_log.dst_port)
        .protocol(conn_log.protocol.clone())
        .build()
        .unwrap();

    let network_connection = NetworkConnectionBuilder::default()
        .state(NetworkConnectionState::Created)
        .src_ip_address(conn_log.src_ip_addr)
        .src_port(conn_log.src_port)
        .dst_ip_address(conn_log.dst_ip_addr)
        .dst_port(conn_log.dst_port)
        .protocol(conn_log.protocol)
        .created_timestamp(conn_log.timestamp)
        .build()
        .unwrap();

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

    // The connection is to a dst ip + port
    graph.add_edge(
        "external_connection",
        outbound.clone_node_key(),
        dst_port.clone_node_key(),
    );

    // There is a network connection between the src and dst ports
    graph.add_edge(
        "connected_to",
        src_port.clone_node_key(),
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

    graph
}

fn handle_inbound_traffic(conn_log: ProcessInboundConnectionLog) -> Graph {
    let mut graph = Graph::new(conn_log.timestamp);

    let asset = AssetBuilder::default()
        .hostname(conn_log.dst_hostname.clone())
        .build()
        .unwrap();

    // A process creates an outbound connection to dst_port
    let process = ProcessBuilder::default()
        .asset_id(conn_log.dst_hostname.clone())
        .state(ProcessState::Existing)
        .process_id(conn_log.pid)
        .last_seen_timestamp(conn_log.timestamp)
        .build()
        .unwrap();

    let inbound = ProcessInboundConnectionBuilder::default()
        .asset_id(conn_log.dst_hostname.clone())
        .state(ProcessInboundConnectionState::Existing)
        .port(conn_log.dst_port)
        .created_timestamp(conn_log.timestamp)
        .build()
        .unwrap();

    let dst_ip = IpAddressBuilder::default()
        .ip_address(conn_log.dst_ip_addr.clone())
        .last_seen_timestamp(conn_log.timestamp)
        .build()
        .unwrap();

    let src_ip = IpAddressBuilder::default()
        .ip_address(conn_log.src_ip_addr.clone())
        .last_seen_timestamp(conn_log.timestamp)
        .build()
        .unwrap();

    let src_port = IpPortBuilder::default()
        .ip_address(conn_log.src_ip_addr.clone())
        .port(conn_log.src_port)
        .build()
        .unwrap();

    let dst_port = IpPortBuilder::default()
        .ip_address(conn_log.dst_ip_addr.clone())
        .port(conn_log.dst_port)
        .build()
        .unwrap();

    let network_connection = NetworkConnectionBuilder::default()
        .state(NetworkConnectionState::Created)
        .src_ip_address(conn_log.src_ip_addr)
        .src_port(conn_log.src_port)
        .dst_ip_address(conn_log.dst_ip_addr)
        .dst_port(conn_log.dst_port)
        .protocol(conn_log.protocol)
        .created_timestamp(conn_log.timestamp)
        .build()
        .unwrap();

    // An asset is assigned an IP
    graph.add_edge(
        "asset_ip",
        asset.clone_node_key(),
        dst_ip.clone_node_key(),
    );

    // A process spawns on an asset
    graph.add_edge(
        "asset_processes",
        asset.clone_node_key(),
        process.clone_node_key(),
    );

    // A process creates a connection
    graph.add_edge(
        "received_connection",
        process.clone_node_key(),
        inbound.clone_node_key(),
    );

    // The connection is over an IP + Port
    graph.add_edge(
        "bound_port",
        inbound.clone_node_key(),
        src_port.clone_node_key(),
    );

    // The connection is to a dst ip + port
    graph.add_edge(
        "external_connection",
        inbound.clone_node_key(),
        dst_port.clone_node_key(),
    );

    // There is a network connection between the src and dst ports
    graph.add_edge(
        "connected_to",
        src_port.clone_node_key(),
        dst_port.clone_node_key(),
    );

    graph.add_node(asset);
    graph.add_node(process);
    graph.add_node(inbound);
    graph.add_node(dst_ip);
    graph.add_node(src_ip);
    graph.add_node(src_port);
    graph.add_node(dst_port);
    graph.add_node(network_connection);

    graph
}

fn handle_process_start(process_start: ProcessStart) -> Graph {
    let mut graph = Graph::new(process_start.timestamp);

    let asset = AssetBuilder::default()
        .hostname(process_start.hostname.clone())
        .build()
        .unwrap();

    let parent = ProcessBuilder::default()
        .hostname(process_start.hostname.clone())
        .state(ProcessState::Existing)
        .process_id(process_start.parent_process_id)
        .last_seen_timestamp(process_start.timestamp)
        .build()
        .unwrap();

    let child = ProcessBuilder::default()
        .hostname(process_start.hostname.clone())
        .process_name(process_start.name)
        .state(ProcessState::Created)
        .process_id(process_start.process_id)
        .created_timestamp(process_start.timestamp)
        .build()
        .unwrap();

    if let Some(exe_path) = process_start.exe {
        let child_exe = FileBuilder::default()
            .hostname(process_start.hostname)
            .state(FileState::Existing)
            .last_seen_timestamp(process_start.timestamp)
            .file_path(exe_path)
            .build()
            .unwrap();

        graph.add_edge("bin_file", child.clone_node_key(), child_exe.clone_node_key());
        info!("child_exe: {}", child_exe.clone().into_json());
        graph.add_node(child_exe);
    }

    graph.add_edge(
        "asset_processes",
        asset.clone_node_key(),
        parent.clone_node_key(),
    );

    graph.add_edge(
        "asset_processes",
        asset.clone_node_key(),
        child.clone_node_key(),
    );

    graph.add_edge("children", parent.clone_node_key(), child.clone_node_key());
    graph.add_node(parent);
    graph.add_node(child);

    graph
}


fn handle_process_stop(process_stop: ProcessStop) -> Graph {
    let terminated_process = ProcessBuilder::default()
        .process_name(process_stop.name)
        .hostname(process_stop.hostname)
        .state(ProcessState::Terminated)
        .process_id(process_stop.process_id)
        .terminated_timestamp(process_stop.timestamp)
        .build()
        .unwrap();

    let mut graph = Graph::new(process_stop.timestamp);
    graph.add_node(terminated_process);

    graph
}


fn handle_file_delete(file_delete: FileDelete) -> Graph {
    let deleter = ProcessBuilder::default()
        .hostname(file_delete.hostname.clone())
        .state(ProcessState::Existing)
        .process_name(file_delete.deleter_process_name)
        .process_id(file_delete.deleter_process_id)
        .last_seen_timestamp(file_delete.timestamp)
        .build()
        .unwrap();

    let file = FileBuilder::default()
        .hostname(file_delete.hostname)
        .state(FileState::Deleted)
        .deleted_timestamp(file_delete.timestamp)
        .file_path(file_delete.path)
        .build()
        .unwrap();

    let mut graph = Graph::new(file_delete.timestamp);

    graph.add_edge("deleted", deleter.clone_node_key(), file.clone_node_key());
    graph.add_node(deleter);
    graph.add_node(file);

    graph
}


fn handle_file_create(file_creator: FileCreate) -> Graph {
    let creator = ProcessBuilder::default()
        .hostname(file_creator.hostname.clone())
        .process_name(file_creator.creator_process_name)
        .state(ProcessState::Existing)
        .process_id(file_creator.creator_process_id)
        .last_seen_timestamp(file_creator.timestamp)
        .build()
        .unwrap();

    let file = FileBuilder::default()
        .hostname(file_creator.hostname)
        .state(FileState::Created)
        .created_timestamp(file_creator.timestamp)
        .file_path(file_creator.path)
        .build()
        .unwrap();

    info!("file {}", file.clone().into_json());

    let mut graph = Graph::new(file_creator.timestamp);

    graph.add_edge("created_files", creator.clone_node_key(), file.clone_node_key());
    graph.add_node(creator);
    graph.add_node(file);

    graph
}

fn handle_file_write(file_write: FileWrite) -> Graph {
    let deleter = ProcessBuilder::default()
        .process_name(file_write.writer_process_name)
        .hostname(file_write.hostname.clone())
        .state(ProcessState::Existing)
        .process_id(file_write.writer_pid)
        .last_seen_timestamp(file_write.timestamp)
        .build()
        .unwrap();

    let file = FileBuilder::default()
        .hostname(file_write.hostname)
        .state(FileState::Existing)
        .last_seen_timestamp(file_write.timestamp)
        .file_path(file_write.path)
        .build()
        .unwrap();

    let mut graph = Graph::new(file_write.timestamp);

    graph.add_edge("wrote_files", deleter.clone_node_key(), file.clone_node_key());
    graph.add_node(deleter);
    graph.add_node(file);

    graph
}

fn handle_file_read(file_read: FileRead) -> Graph {
    let deleter = ProcessBuilder::default()
        .process_name(file_read.reader_process_name)
        .hostname(file_read.hostname.clone())
        .state(ProcessState::Existing)
        .process_id(file_read.reader_process_id)
        .last_seen_timestamp(file_read.timestamp)
        .build()
        .unwrap();

    let file = FileBuilder::default()
        .hostname(file_read.hostname)
        .state(FileState::Existing)
        .last_seen_timestamp(file_read.timestamp)
        .file_path(file_read.path)
        .build()
        .unwrap();

    let mut graph = Graph::new(file_read.timestamp);

    graph.add_edge("read_files", deleter.clone_node_key(), file.clone_node_key());
    graph.add_node(deleter);
    graph.add_node(file);

    graph
}

fn handle_log(raw_log: Value) -> Result<Graph, Error> {
    let sourcetype = match raw_log
        .get("sourcetype")
        .and_then(|sourcetype| sourcetype.as_str())
    {
        Some(sourcetype) => sourcetype.to_owned(),
        None => bail!("Sourcetype must be specified and a valid string"),
    };

    info!("Parsing log of type: {}", sourcetype);
    let graph = match &sourcetype[..] {
        "FILE_READ" => handle_file_read(serde_json::from_value(raw_log)?),
        "FILE_WRITE" => handle_file_write(serde_json::from_value(raw_log)?),
        "FILE_CREATE" => handle_file_create(serde_json::from_value(raw_log)?),
        "FILE_DELETE" => handle_file_delete(serde_json::from_value(raw_log)?),
        "PROCESS_START" => handle_process_start(serde_json::from_value(raw_log)?),
        "PROCESS_STOP" => handle_process_stop(serde_json::from_value(raw_log)?),
        "INBOUND_TCP" => handle_inbound_traffic(serde_json::from_value(raw_log)?),
        "OUTBOUND_TCP" => handle_outbound_traffic(serde_json::from_value(raw_log)?),
        _ => bail!("invalid sourcetype"),
    };

    Ok(graph)
}

#[derive(Clone)]
struct GenericSubgraphGenerator<S>
    where S: S3
{
    s3: Arc<S>,
}

impl<S> EventHandler<Vec<serde_json::Value>> for GenericSubgraphGenerator<S>
    where S: S3
{
    fn handle_event(&self, event: Vec<serde_json::Value>) -> Result<(), Error> {
        let subgraphs: Vec<_> = event
            .into_iter()
            .map(handle_log)
            .map(|res| {
                if let Err(ref e) = res {
                    error!("Failed to generate subgraph with: {}", e);
                }
                res
            })
            .flat_map(Result::ok)
            .collect();

        upload_subgraphs(self.s3.as_ref(), GeneratedSubgraphs::new(subgraphs))?;
        Ok(())
    }
}

fn my_handler(event: SqsEvent, ctx: Context) -> Result<(), HandlerError> {
    let region = {
        let region_str = std::env::var("AWS_REGION").expect("AWS_REGION");
        Region::from_str(&region_str).expect("Invalid Region")
    };
    info!("Creating sqs_client");
    let sqs_client = Arc::new(SqsClient::new(region.clone()));

    info!("Creating s3_client");
    let s3_client = Arc::new(S3Client::new(region.clone()));

    info!("Creating retriever");
    let retriever = S3EventRetriever::new(
        s3_client.clone(),
        |d| {
            info!("Parsing: {:?}", d);
            events_from_s3_sns_sqs(d)
        },
        ZstdJsonDecoder {
            buffer: Vec::with_capacity(1 << 8),
        },
    );

    let queue_url = std::env::var("QUEUE_URL").expect("QUEUE_URL");

    info!("Creating sqs_completion_handler");
    let sqs_completion_handler = BlockingSqsCompletionHandler::new(sqs_client, queue_url);

    let handler = GenericSubgraphGenerator {s3: s3_client};

    let mut sqs_service = SqsService::new(retriever, handler, sqs_completion_handler);

    info!("Handing off event");
    sqs_service.run(event, ctx)?;

    Ok(())
}

fn main() {
    simple_logger::init_with_level(log::Level::Info).unwrap();
    lambda!(my_handler);
}
