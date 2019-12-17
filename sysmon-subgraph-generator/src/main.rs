extern crate aws_lambda_events;
extern crate chrono;
extern crate failure;
extern crate futures;
extern crate graph_descriptions;
extern crate graph_generator_lib;
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
use sqs_lambda::EventHandler;
use sqs_lambda::events_from_s3_sns_sqs;
use sqs_lambda::NopSqsCompletionHandler;
use sqs_lambda::S3EventRetriever;
use sqs_lambda::SqsService;
use sqs_lambda::ZstdDecoder;
use sysmon::*;
use uuid::Uuid;

use graph_descriptions::*;
use graph_descriptions::file::FileState;
use graph_descriptions::graph_description::*;
use graph_descriptions::network_connection::NetworkConnectionState;
use graph_descriptions::node::NodeT;
use graph_descriptions::process::ProcessState;
use graph_descriptions::process_inbound_connection::ProcessInboundConnectionState;
use graph_descriptions::process_outbound_connection::ProcessOutboundConnectionState;
use graph_generator_lib::upload_subgraphs;

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
    image_path.split("\\").last().map(|name| {
        name.replace("- ", "").replace("\\", "")
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

    Ok(ts as u64)
}

fn handle_process_start(process_start: &ProcessCreateEvent) -> Result<Graph, Error> {
    let timestamp = utc_to_epoch(&process_start.event_data.utc_time)?;
    let mut graph = Graph::new(timestamp);

    let asset = AssetBuilder::default()
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
        .created_timestamp(process_start.event_data.parent_process_guid.get_creation_timestamp())
        .build()
        .unwrap();

    let child = ProcessBuilder::default()
        .asset_id(process_start.system.computer.computer.clone())
        .process_name(get_image_name(&process_start.event_data.image.clone()).unwrap())
        .process_command_line(&process_start.event_data.command_line.command_line)
        .state(ProcessState::Created)
        .process_id(process_start.event_data.process_id)
        .created_timestamp(timestamp)
        .build()
        .unwrap();

    let child_exe = FileBuilder::default()
        .asset_id(process_start.system.computer.computer.clone())
        .state(FileState::Existing)
        .last_seen_timestamp(timestamp)
        .file_path(strip_file_zone_identifier(&process_start.event_data.image))
        .build()
        .unwrap();

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
        .hostname(conn_log.system.computer.computer.clone())
        .build()
        .unwrap();

    // A process creates an outbound connection to dst_port
    let process = ProcessBuilder::default()
        .hostname(conn_log.system.computer.computer.clone())
        .state(ProcessState::Existing)
        .process_id(conn_log.event_data.process_id)
        .last_seen_timestamp(timestamp)
        .build()
        .unwrap();

    let outbound = ProcessOutboundConnectionBuilder::default()
        .hostname(conn_log.system.computer.computer.clone())
        .state(ProcessOutboundConnectionState::Connected)
        .port(conn_log.event_data.source_port)
        .created_timestamp(timestamp)
        .build()
        .unwrap();

    let src_ip = IpAddressBuilder::default()
        .ip_address(conn_log.event_data.source_ip.clone())
        .last_seen_timestamp(timestamp)
        .build()
        .unwrap();

    let dst_ip = IpAddressBuilder::default()
        .ip_address(conn_log.event_data.destination_ip.clone())
        .last_seen_timestamp(timestamp)
        .build()
        .unwrap();

    let src_port = IpPortBuilder::default()
        .ip_address(conn_log.event_data.source_ip.clone())
        .port(conn_log.event_data.source_port)
        .protocol(conn_log.event_data.protocol.clone())
        .build()
        .unwrap();

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
        .protocol(conn_log.event_data.protocol)
        .created_timestamp(timestamp)
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

    Ok(graph)
}

// Inbound is the 'src' in sysmon
fn handle_inbound_connection(conn_log: &NetworkEvent) -> Result<Graph, Error> {
    let timestamp = utc_to_epoch(&conn_log.event_data.utc_time)?;

    let mut graph = Graph::new(timestamp);

    let asset = AssetBuilder::default()
        .hostname(conn_log.system.computer.computer.clone())
        .build()
        .unwrap();

    // A process creates an outbound connection to dst_port
    let process = ProcessBuilder::default()
        .hostname(conn_log.system.computer.computer.clone())
        .state(ProcessState::Existing)
        .process_id(conn_log.event_data.process_id)
        .last_seen_timestamp(timestamp)
        .build()
        .unwrap();

    let outbound = ProcessInboundConnectionBuilder::default()
        .hostname(conn_log.system.computer.computer.clone())
        .state(ProcessInboundConnectionState::Bound)
        .port(conn_log.event_data.source_port)
        .created_timestamp(timestamp)
        .build()
        .unwrap();

    let src_ip = IpAddressBuilder::default()
        .ip_address(conn_log.event_data.source_ip.clone())
        .last_seen_timestamp(timestamp)
        .build()
        .unwrap();

    let dst_ip = IpAddressBuilder::default()
        .ip_address(conn_log.event_data.destination_ip.clone())
        .last_seen_timestamp(timestamp)
        .build()
        .unwrap();

    let src_port = IpPortBuilder::default()
        .ip_address(conn_log.event_data.source_ip.clone())
        .port(conn_log.event_data.source_port)
        .protocol(conn_log.event_data.protocol.clone())
        .build()
        .unwrap();

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
        .created_timestamp(timestamp)
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
        .created_timestamp(file_create.event_data.process_guid.get_creation_timestamp())
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


struct SysmonSubgraphGenerator<S>
    where S: (Fn(GeneratedSubgraphs) -> Result<(), Error>) + Clone
{
    output_handler: S,

}

impl<S> Clone for SysmonSubgraphGenerator<S>
    where S: (Fn(GeneratedSubgraphs) -> Result<(), Error>) + Clone
{
    fn clone(&self) -> Self {
        Self {
            output_handler: self.output_handler.clone(),
        }
    }
}


impl<S> SysmonSubgraphGenerator<S>
    where S: (Fn(GeneratedSubgraphs) -> Result<(), Error>) + Clone
{
    pub fn new(output_handler: S) -> Self {
        Self {
            output_handler
        }
    }
}

impl<S> EventHandler<Vec<u8>> for SysmonSubgraphGenerator<S>
    where S: (Fn(GeneratedSubgraphs) -> Result<(), Error>) + Clone
{
    fn handle_event(&self, event: Vec<u8>) -> Result<(), Error> {
        info!("Handling raw event");

        let events: Vec<_> = log_time!(
            "event split",
            event.split(|i| &[*i][..] == &b"\n"[..]).collect()
        );

        let subgraphs: Vec<_> = log_time!(
            "events par_iter",
             events.into_par_iter().flat_map(move |event| {
                let event = String::from_utf8_lossy(event);
                let event = Event::from_str(&event);
                let event = event.ok()?;

                match event {
                    Event::ProcessCreate(event) => {
                        info!("Handling process create");

                        match handle_process_start(&event) {
                            Ok(event) => Some(event),
                            Err(e) => {
                                warn!("Failed to process process start event: {}", e);
                                None
                            }
                        }
                    }
                    Event::FileCreate(event) => {
                        info!("FileCreate");
                                                unimplemented!()

//                        match handle_file_create(&event) {
//                            Ok(event) => Some(event),
//                            Err(e) => {
//                                warn!("Failed to process file create event: {}", e);
//                                None
//                            }
//                        }
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
                        unimplemented!()
//                        match handle_outbound_connection(&event) {
//                            Ok(event) => Some(event),
//                            Err(e) => {
//                                warn!("Failed to process outbound network event: {}", e);
//                                None
//                            }
//                        }
                    }
                    catch => {warn!("Unsupported event_type: {:?}", catch); None}
                }
            }).collect()
        );

        info!("Completed mapping {} subgraphs", subgraphs.len());
        let graphs = GeneratedSubgraphs { subgraphs };

        log_time!(
            "upload_subgraphs",
            (self.output_handler)(graphs)
        )?;

        Ok(())
    }
}


fn my_handler(event: SqsEvent, ctx: Context) -> Result<(), HandlerError> {
    let region = {
        let region_str = std::env::var("AWS_REGION").expect("AWS_REGION");
        Region::from_str(&region_str).expect("Region error")
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
        ZstdDecoder { buffer: Vec::with_capacity(1 << 8) },
    );

    let queue_url = std::env::var("QUEUE_URL").expect("QUEUE_URL");

    info!("Creating sqs_completion_handler");
    let sqs_completion_handler = NopSqsCompletionHandler::new(
        queue_url
    );

    let handler = SysmonSubgraphGenerator::new(
        move |generated_subgraphs| {
            upload_subgraphs(s3_client.as_ref(), generated_subgraphs)
        }
    );

    let mut sqs_service = SqsService::new(
        retriever,
        handler,
        sqs_completion_handler,
    );

    info!("Handing off event");
    sqs_service.run(event, ctx)?;

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    simple_logger::init_with_level(log::Level::Info).unwrap();

    info!("Starting lambda");
    lambda!(my_handler);
    Ok(())
}


#[cfg(test)]
mod tests {
    use std::time::Duration;

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

        let handler = SysmonSubgraphGenerator::new(
            move |generated_subgraphs| {
                println!("generated subgraphs");
                Ok(())
            }
        );

        handler.handle_event(vec![]).expect("handle_event failed");
    }
}

