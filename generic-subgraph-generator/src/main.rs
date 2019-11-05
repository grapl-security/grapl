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
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate aws_lambda_events;
extern crate rusoto_core;
extern crate rusoto_s3;
extern crate rusoto_sqs;
extern crate serde_json;
extern crate simple_logger;
extern crate sqs_lambda;
extern crate uuid;

use std::str::FromStr;
use sqs_lambda::events_from_s3_sns_sqs;
use sqs_lambda::BlockingSqsCompletionHandler;
use sqs_lambda::EventHandler;
use sqs_lambda::S3EventRetriever;
use sqs_lambda::SqsService;
use sqs_lambda::ZstdJsonDecoder;
use std::marker::PhantomData;

use lambda::error::HandlerError;
use lambda::lambda;
use lambda::Context;
use lambda::Handler;

use aws_lambda_events::event::sqs::{SqsEvent, SqsMessage};

use rusoto_core::Region;
use rusoto_s3::S3Client;
use rusoto_s3::{GetObjectRequest, S3};
use rusoto_sqs::{GetQueueUrlRequest, Sqs, SqsClient};

use failure::Error;
use graph_descriptions::graph_description::*;
use graph_descriptions::*;
use graph_generator_lib::upload_subgraphs;
use regex::Regex;
use serde_json::Value;
use std::sync::Arc;

#[derive(Serialize, Deserialize)]
pub struct ProcessStart {
    pid: u64,
    ppid: u64,
    name: String,
    ip: Option<String>,
    hostname: Option<String>,
    asset_id: String,
    arguments: String,
    timestamp: u64,
    exe: Option<String>,
    sourcetype: String,
}

#[derive(Serialize, Deserialize)]
pub struct ProcessStop {
    pid: u64,
    name: String,
    ip: Option<String>,
    hostname: Option<String>,
    asset_id: String,
    timestamp: u64,
    sourcetype: String,
}

#[derive(Serialize, Deserialize)]
pub struct FileCreate {
    creator_pid: u64,
    creator_name: String,
    path: String,
    ip: Option<String>,
    hostname: Option<String>,
    asset_id: String,
    timestamp: u64,
    sourcetype: String,
}

#[derive(Serialize, Deserialize)]
pub struct FileDelete {
    deleter_pid: u64,
    deleter_name: String,
    path: String,
    ip: Option<String>,
    hostname: Option<String>,
    asset_id: String,
    timestamp: u64,
    sourcetype: String,
}

#[derive(Serialize, Deserialize)]
pub struct FileRead {
    reader_pid: u64,
    reader_name: String,
    path: String,
    ip: Option<String>,
    hostname: Option<String>,
    asset_id: String,
    timestamp: u64,
    sourcetype: String,
}

#[derive(Serialize, Deserialize)]
pub struct FileWrite {
    writer_pid: u64,
    writer_name: String,
    path: String,
    ip: Option<String>,
    hostname: Option<String>,
    asset_id: String,
    timestamp: u64,
    sourcetype: String,
}

#[derive(Serialize, Deserialize)]
pub struct OutboundConnectionLog {
    pid: u64,
    protocol: String,
    src_port: u32,
    dst_port: u32,
    src_addr: String,
    dst_addr: String,
    src_asset_id: Option<String>,
    dst_asset_id: Option<String>,
    timestamp: u64,
    sourcetype: String,
}

#[derive(Serialize, Deserialize)]
pub struct InboundConnectionLog {
    pid: u64,
    protocol: String,
    src_port: u32,
    dst_port: u32,
    src_addr: String,
    dst_addr: String,
    src_asset_id: Option<String>,
    dst_asset_id: Option<String>,
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

fn handle_outbound_traffic(conn_log: OutboundConnectionLog) -> GraphDescription {
    let mut graph = GraphDescription::new(conn_log.timestamp);

    // A process creates an outbound connection to dst_port
    // Another process must have an inbound connection to src_port
    // Or the other process is external/ not running the instrumentation
    let process = ProcessDescriptionBuilder::default()
        .asset_id(conn_log.src_asset_id.clone())
        .host_ip(conn_log.src_addr.to_owned())
        .state(ProcessState::Existing)
        .process_id(conn_log.pid)
        .last_seen_timestamp(conn_log.timestamp)
        .build()
        .unwrap();

    let outbound = OutboundConnectionBuilder::default()
        .asset_id(conn_log.src_asset_id)
        .host_ip(conn_log.src_addr.to_owned())
        .state(ConnectionState::Created)
        .port(conn_log.src_port)
        .created_timestamp(conn_log.timestamp)
        .build()
        .unwrap();

    if is_internal_ip(&conn_log.dst_addr.to_owned()) {
        let inbound = InboundConnectionBuilder::default()
            .asset_id(conn_log.dst_asset_id)
            .host_ip(conn_log.dst_addr.to_owned())
            .state(ConnectionState::Existing)
            .port(conn_log.dst_port)
            .last_seen_timestamp(conn_log.timestamp)
            .build()
            .unwrap();

        graph.add_edge("connection", outbound.clone_key(), inbound.clone_key());
        graph.add_node(inbound);
    } else {
        let external_ip = IpAddressDescription::new(
            conn_log.timestamp,
            conn_log.dst_addr.to_owned(),
            "TCP",
        );

        graph.add_edge(
            "external_connection",
            outbound.clone_key(),
            external_ip.clone_key(),
        );

        graph.add_node(external_ip);
    }

    graph.add_edge(
        "created_connection",
        process.clone_key(),
        outbound.clone_key(),
    );

    graph.add_node(outbound);
    graph.add_node(process);

    graph
}

fn handle_inbound_traffic(conn_log: InboundConnectionLog) -> GraphDescription {
    let mut graph = GraphDescription::new(conn_log.timestamp);

    let process = ProcessDescriptionBuilder::default()
        .asset_id(conn_log.src_asset_id.clone())
        .host_ip(conn_log.src_addr.clone())
        .state(ProcessState::Existing)
        .process_id(conn_log.pid)
        .last_seen_timestamp(conn_log.timestamp)
        .build()
        .unwrap();

    // Inbound is the 'src', at least in sysmon
    let inbound = InboundConnectionBuilder::default()
        .asset_id(conn_log.src_asset_id.clone())
        .host_ip(conn_log.src_addr.clone())
        .state(ConnectionState::Created)
        .port(conn_log.src_port)
        .created_timestamp(conn_log.timestamp)
        .build()
        .unwrap();

    if is_internal_ip(&conn_log.dst_addr.clone()) {
        let outbound = InboundConnectionBuilder::default()
            .asset_id(conn_log.dst_asset_id.clone())
            .host_ip(conn_log.dst_addr.clone())
            .state(ConnectionState::Created)
            .port(conn_log.src_port)
            .last_seen_timestamp(conn_log.timestamp)
            .build()
            .unwrap();

        graph.add_edge("connection", outbound.clone_key(), inbound.clone_key());

        graph.add_node(outbound);
    } else {
        let external_ip =
            IpAddressDescription::new(
                conn_log.timestamp,
                conn_log.dst_addr.clone(),
                "TCP"
            );

        graph.add_edge(
            "external_connection",
            inbound.clone_key(),
            external_ip.clone_key(),
        );

        graph.add_node(external_ip);
    }

    graph.add_edge("bound_connection", process.clone_key(), inbound.clone_key());

    graph.add_node(inbound);
    graph.add_node(process);

    println!("generated inbound subgraph {:#?}", graph);
    graph
}

fn handle_process_start(process_start: ProcessStart) -> GraphDescription {
    let mut graph = GraphDescription::new(process_start.timestamp);

    let parent = ProcessDescriptionBuilder::default()
        .asset_id(process_start.asset_id.clone())
        .host_ip(process_start.ip.clone())
        .hostname(process_start.hostname.clone())
        .state(ProcessState::Existing)
        .process_id(process_start.ppid)
        .last_seen_timestamp(process_start.timestamp)
        .build()
        .unwrap();

    let child = ProcessDescriptionBuilder::default()
        .asset_id(process_start.asset_id.clone())
        .host_ip(process_start.ip.clone())
        .hostname(process_start.hostname.clone())
        .process_name(process_start.name)
        .state(ProcessState::Created)
        .process_id(process_start.pid)
        .created_timestamp(process_start.timestamp)
        .build()
        .unwrap();

    if let Some(exe_path) = process_start.exe {
        let child_exe = FileDescriptionBuilder::default()
            .asset_id(process_start.asset_id)
            .host_ip(process_start.ip)
            .hostname(process_start.hostname)
            .state(FileState::Existing)
            .last_seen_timestamp(process_start.timestamp)
            .file_path(exe_path)
            .build()
            .unwrap();

        graph.add_edge("bin_file", child.clone_key(), child_exe.clone_key());
        info!("child_exe: {}", child_exe.clone().into_json());
        graph.add_node(child_exe);
    }

    info!("parent: {}", parent.clone().into_json());
    info!("child: {}", child.clone().into_json());

    graph.add_edge("children", parent.clone_key(), child.clone_key());
    graph.add_node(parent);
    graph.add_node(child);

    graph
}

fn handle_process_stop(process_stop: ProcessStop) -> GraphDescription {
    let terminated_process = ProcessDescriptionBuilder::default()
        .process_name(process_stop.name)
        .asset_id(process_stop.asset_id)
        .host_ip(process_stop.ip)
        .hostname(process_stop.hostname)
        .state(ProcessState::Terminated)
        .process_id(process_stop.pid)
        .terminated_timestamp(process_stop.timestamp)
        .build()
        .unwrap();

    let mut graph = GraphDescription::new(process_stop.timestamp);
    graph.add_node(terminated_process);

    graph
}

fn handle_file_delete(file_delete: FileDelete) -> GraphDescription {
    let deleter = ProcessDescriptionBuilder::default()
        .asset_id(file_delete.asset_id.clone())
        .host_ip(file_delete.ip.clone())
        .hostname(file_delete.hostname.clone())
        .state(ProcessState::Existing)
        .process_name(file_delete.deleter_name)
        .process_id(file_delete.deleter_pid)
        .last_seen_timestamp(file_delete.timestamp)
        .build()
        .unwrap();

    let file = FileDescriptionBuilder::default()
        .asset_id(file_delete.asset_id)
        .host_ip(file_delete.ip)
        .hostname(file_delete.hostname)
        .state(FileState::Deleted)
        .deleted_timestamp(file_delete.timestamp)
        .file_path(file_delete.path)
        .build()
        .unwrap();

    let mut graph = GraphDescription::new(file_delete.timestamp);

    graph.add_edge("deleted", deleter.clone_key(), file.clone_key());
    graph.add_node(deleter);
    graph.add_node(file);

    graph
}

fn handle_file_create(file_creator: FileCreate) -> GraphDescription {
    let creator = ProcessDescriptionBuilder::default()
        .asset_id(file_creator.asset_id.clone())
        .host_ip(file_creator.ip.clone())
        .hostname(file_creator.hostname.clone())
        .process_name(file_creator.creator_name)
        .state(ProcessState::Existing)
        .process_id(file_creator.creator_pid)
        .last_seen_timestamp(file_creator.timestamp)
        .build()
        .unwrap();

    let file = FileDescriptionBuilder::default()
        .asset_id(file_creator.asset_id)
        .host_ip(file_creator.ip)
        .hostname(file_creator.hostname)
        .state(FileState::Created)
        .created_timestamp(file_creator.timestamp)
        .file_path(file_creator.path)
        .build()
        .unwrap();

    info!("file {}", file.clone().into_json());

    let mut graph = GraphDescription::new(file_creator.timestamp);

    graph.add_edge("created_files", creator.clone_key(), file.clone_key());
    graph.add_node(creator);
    graph.add_node(file);

    graph
}

fn handle_file_write(file_write: FileWrite) -> GraphDescription {
    let deleter = ProcessDescriptionBuilder::default()
        .process_name(file_write.writer_name)
        .asset_id(file_write.asset_id.clone())
        .host_ip(file_write.ip.clone())
        .hostname(file_write.hostname.clone())
        .state(ProcessState::Existing)
        .process_id(file_write.writer_pid)
        .last_seen_timestamp(file_write.timestamp)
        .build()
        .unwrap();

    let file = FileDescriptionBuilder::default()
        .asset_id(file_write.asset_id)
        .host_ip(file_write.ip)
        .hostname(file_write.hostname)
        .state(FileState::Existing)
        .last_seen_timestamp(file_write.timestamp)
        .file_path(file_write.path)
        .build()
        .unwrap();

    let mut graph = GraphDescription::new(file_write.timestamp);

    graph.add_edge("wrote_files", deleter.clone_key(), file.clone_key());
    graph.add_node(deleter);
    graph.add_node(file);

    graph
}

fn handle_file_read(file_read: FileRead) -> GraphDescription {
    let deleter = ProcessDescriptionBuilder::default()
        .process_name(file_read.reader_name)
        .asset_id(file_read.asset_id.clone())
        .host_ip(file_read.ip.clone())
        .hostname(file_read.hostname.clone())
        .state(ProcessState::Existing)
        .process_id(file_read.reader_pid)
        .last_seen_timestamp(file_read.timestamp)
        .build()
        .unwrap();

    let file = FileDescriptionBuilder::default()
        .asset_id(file_read.asset_id)
        .host_ip(file_read.ip)
        .hostname(file_read.hostname)
        .state(FileState::Existing)
        .last_seen_timestamp(file_read.timestamp)
        .file_path(file_read.path)
        .build()
        .unwrap();

    let mut graph = GraphDescription::new(file_read.timestamp);

    graph.add_edge("read_files", deleter.clone_key(), file.clone_key());
    graph.add_node(deleter);
    graph.add_node(file);

    graph
}

fn handle_log(raw_log: Value) -> Result<GraphDescription, Error> {
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
