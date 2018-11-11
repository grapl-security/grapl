#![feature(nll)]
#[macro_use]
extern crate failure;
extern crate graph_descriptions;
extern crate graph_generator_lib;
#[macro_use]
extern crate lazy_static;
#[macro_use] extern crate log;
extern crate regex;
extern crate serde;
#[macro_use] extern crate serde_derive;
extern crate serde_json;
extern crate uuid;

use failure::Error;
use graph_descriptions::*;
use graph_descriptions::graph_description::*;
use graph_generator_lib::handle_json_encoded_logs;
use regex::bytes::Regex;
use serde_json::Value;


#[derive(Serialize, Deserialize)]
pub struct OutboundConnectionLog {
    pid: u64,
    protocol: String,
    src_port: u32,
    dst_port: u32,
    src_addr: String,
    dst_addr: String,
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
    timestamp: u64,
    sourcetype: String,
}

fn is_internal_ip(ip: &[u8]) -> bool {

    lazy_static!(
        static ref RE: Regex = Regex::new(
            r"/(^127\.)|(^192\.168\.)|(^10\.)|(^172\.1[6-9]\.)|(^172\.2[0-9]\.)|(^172\.3[0-1]\.)|(^::1$)|(^[fF][cCdD])/"
        ).expect("is_internal_ip regex");
    );

    RE.is_match(ip)
}


fn handle_outbound_traffic(conn_log: OutboundConnectionLog) -> GraphDescription {
    let mut graph = GraphDescription::new(
        conn_log.timestamp
    );

    // A process creates an outbound connection to dst_port
    // Another process must have an inbound connection to src_port
    // Or the other process is external/ not running the instrumentation
    let process = ProcessDescriptionBuilder::default()
        .host_ip(conn_log.src_addr.clone().into_bytes())
        .state(ProcessState::Existing)
        .pid(conn_log.pid)
        .timestamp(conn_log.timestamp)
        .build()
        .unwrap();

    let outbound = OutboundConnectionBuilder::default()
        .host_ip(conn_log.src_addr.clone().into_bytes())
        .state(ConnectionState::Created)
        .port(conn_log.src_port)
        .timestamp(conn_log.timestamp)
        .build()
        .unwrap();


    if is_internal_ip(&conn_log.dst_addr.clone().into_bytes()) {
        let inbound = InboundConnectionBuilder::default()
            .host_ip(conn_log.dst_addr.clone().into_bytes())
            .state(ConnectionState::Existing)
            .port(conn_log.dst_port)
            .timestamp(conn_log.timestamp)
            .build()
            .unwrap();

        graph.add_edge("connection",
                       outbound.clone_key(),
                       inbound.clone_key());
        graph.add_node(inbound);
    } else {
        let external_ip = IpAddressDescription::new(
            conn_log.timestamp,
            conn_log.dst_addr.clone().into_bytes()
        );

        graph.add_edge("external_connection",
                       outbound.clone_key(),
                       external_ip.clone_key());

        graph.add_node(external_ip);
    }

    graph.add_edge("created_connection",
                   process.clone_key(),
                   outbound.clone_key());


    graph.add_node(outbound);
    graph.add_node(process);

    graph
}

fn handle_inbound_traffic(conn_log: OutboundConnectionLog) -> GraphDescription {
    let mut graph = GraphDescription::new(
        conn_log.timestamp
    );

    let process = ProcessDescriptionBuilder::default()
        .host_ip(conn_log.src_addr.clone().into_bytes())
        .state(ProcessState::Existing)
        .pid(conn_log.pid)
        .timestamp(conn_log.timestamp)
        .build()
        .unwrap();

    // Inbound is the 'src', at least in sysmon
    let inbound = InboundConnectionBuilder::default()
        .host_ip(conn_log.src_addr.clone().into_bytes())
        .state(ConnectionState::Created)
        .port(conn_log.src_port)
        .timestamp(conn_log.timestamp)
        .build()
        .unwrap();


    if is_internal_ip(&conn_log.dst_addr.clone().into_bytes()) {
        let outbound = InboundConnectionBuilder::default()
            .host_ip(conn_log.dst_addr.clone().into_bytes())
            .state(ConnectionState::Created)
            .port(conn_log.src_port)
            .timestamp(conn_log.timestamp)
            .build()
            .unwrap();

        graph.add_edge("connection",
                       outbound.clone_key(),
                       inbound.clone_key());

        graph.add_node(outbound);
    } else {
        let external_ip = IpAddressDescription::new(
            conn_log.timestamp,
            conn_log.dst_addr.clone().into_bytes()
        );

        graph.add_edge("external_connection",
                       inbound.clone_key(),
                       external_ip.clone_key());

        graph.add_node(external_ip);
    }

    graph.add_edge("bound_connection",
                   process.clone_key(),
                   inbound.clone_key());

    graph.add_node(inbound);
    graph.add_node(process);

    graph
}


#[derive(Serialize, Deserialize)]
pub struct ProcessStart {
    pid: u64,
    ppid: u64,
    name: String,
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
    asset_id: String,
    timestamp: u64,
    sourcetype: String,
}

#[derive(Serialize, Deserialize)]
pub struct FileCreate {
    creator_pid: u64,
    creator_name: String,
    path: String,
    asset_id: String,
    timestamp: u64,
    sourcetype: String,
}

#[derive(Serialize, Deserialize)]
pub struct FileDelete {
    deleter_pid: u64,
    deleter_name: String,
    path: String,
    asset_id: String,
    timestamp: u64,
    sourcetype: String,
}

#[derive(Serialize, Deserialize)]
pub struct FileRead {
    reader_pid: u64,
    reader_name: String,
    path: String,
    asset_id: String,
    timestamp: u64,
    sourcetype: String,
}

#[derive(Serialize, Deserialize)]
pub struct FileWrite {
    writer_pid: u64,
    writer_name: String,
    path: String,
    asset_id: String,
    timestamp: u64,
    sourcetype: String,
}

fn handle_process_start(process_start: ProcessStart) -> GraphDescription {
    let mut graph = GraphDescription::new(
        process_start.timestamp
    );

    let parent = ProcessDescriptionBuilder::default()
        .asset_id(process_start.asset_id.clone())
        .state(ProcessState::Existing)
        .pid(process_start.ppid)
        .timestamp(process_start.timestamp)
        .build()
        .unwrap();

    let child = ProcessDescriptionBuilder::default()
        .asset_id(process_start.asset_id.clone())
        .state(ProcessState::Created)
        .pid(process_start.pid)
        .timestamp(process_start.timestamp)
        .build()
        .unwrap();


    if let Some(exe_path) = process_start.exe {

        let child_exe = FileDescriptionBuilder::default()
            .asset_id(process_start.asset_id)
            .state(FileState::Existing)
            .timestamp(process_start.timestamp)
            .path(exe_path)
            .build()
            .unwrap();

        graph.add_edge("bin_file",
            child.clone_key(),
            child_exe.clone_key()
        );
        info!("child_exe: {}", child_exe.clone().into_json());
        graph.add_node(child_exe);
    }

    info!("parent: {}", parent.clone().into_json());
    info!("child: {}", child.clone().into_json());

    graph.add_edge("children",
                   parent.clone_key(),
                   child.clone_key());
    graph.add_node(parent);
    graph.add_node(child);


    graph
}

fn handle_process_stop(process_stop: ProcessStop) -> GraphDescription {
    let terminated_process = ProcessDescriptionBuilder::default()
        .asset_id(process_stop.asset_id.clone())
        .state(ProcessState::Terminated)
        .pid(process_stop.pid)
        .timestamp(process_stop.timestamp)
        .build()
        .unwrap();

    let mut graph = GraphDescription::new(
        process_stop.timestamp
    );
    graph.add_node(terminated_process);

    graph
}

fn handle_file_delete(file_delete: FileDelete) -> GraphDescription {
    let deleter = ProcessDescriptionBuilder::default()
        .asset_id(file_delete.asset_id.clone())
        .state(ProcessState::Existing)
        .pid(file_delete.deleter_pid)
        .timestamp(file_delete.timestamp)
        .build()
        .unwrap();

    let file = FileDescriptionBuilder::default()
        .asset_id(file_delete.asset_id)
        .state(FileState::Deleted)
        .timestamp(file_delete.timestamp)
        .path(file_delete.path)
        .build()
        .unwrap();

    let mut graph = GraphDescription::new(
        file_delete.timestamp
    );

    graph.add_edge("deleted",
                   deleter.clone_key(),
                   file.clone_key());
    graph.add_node(deleter);
    graph.add_node(file);

    graph
}

fn handle_file_create(file_creator: FileCreate) -> GraphDescription {
    let creator = ProcessDescriptionBuilder::default()
        .asset_id(file_creator.asset_id.clone())
        .state(ProcessState::Existing)
        .pid(file_creator.creator_pid)
        .timestamp(file_creator.timestamp)
        .build()
        .unwrap();

    let file = FileDescriptionBuilder::default()
        .asset_id(file_creator.asset_id)
        .state(FileState::Created)
        .timestamp(file_creator.timestamp)
        .path(file_creator.path)
        .build()
        .unwrap();

    info!("file {}", file.clone().into_json());

    let mut graph = GraphDescription::new(
        file_creator.timestamp
    );

    graph.add_edge("created_files",
                   creator.clone_key(),
                   file.clone_key());
    graph.add_node(creator);
    graph.add_node(file);

    graph
}

fn handle_file_write(file_write: FileWrite) -> GraphDescription {
    let deleter = ProcessDescriptionBuilder::default()
        .asset_id(file_write.asset_id.clone())
        .state(ProcessState::Existing)
        .pid(file_write.writer_pid)
        .timestamp(file_write.timestamp)
        .build()
        .unwrap();

    let file = FileDescriptionBuilder::default()
        .asset_id(file_write.asset_id)
        .state(FileState::Existing)
        .timestamp(file_write.timestamp)
        .path(file_write.path)
        .build()
        .unwrap();

    let mut graph = GraphDescription::new(
        file_write.timestamp
    );

    graph.add_edge("wrote_files",
                   deleter.clone_key(),
                   file.clone_key());
    graph.add_node(deleter);
    graph.add_node(file);

    graph
}

fn handle_file_read(file_read: FileRead) -> GraphDescription {
    let deleter = ProcessDescriptionBuilder::default()
        .asset_id(file_read.asset_id.clone())
        .state(ProcessState::Existing)
        .pid(file_read.reader_pid)
        .timestamp(file_read.timestamp)
        .build()
        .unwrap();

    let file = FileDescriptionBuilder::default()
        .asset_id(file_read.asset_id)
        .state(FileState::Existing)
        .timestamp(file_read.timestamp)
        .path(file_read.path)
        .build()
        .unwrap();

    let mut graph = GraphDescription::new(
        file_read.timestamp
    );

    graph.add_edge("read_files",
                   deleter.clone_key(),
                   file.clone_key());
    graph.add_node(deleter);
    graph.add_node(file);

    graph
}

fn handle_log(raw_log: Value) -> Result<GraphDescription, Error> {
    let sourcetype = raw_log["sourcetype"].as_str().unwrap();

    info!("Parsing log of type: {}", sourcetype);
    let graph = match sourcetype {
        "FILE_READ" => handle_file_read(serde_json::from_value(raw_log)?),
        "FILE_WRITE" => handle_file_write(serde_json::from_value(raw_log)?),
        "FILE_CREATE" => handle_file_create(serde_json::from_value(raw_log)?),
        "FILE_DELETE" => handle_file_delete(serde_json::from_value(raw_log)?),
        "PROCESS_START" => handle_process_start(serde_json::from_value(raw_log)?),
        "PROCESS_STOP" => handle_process_stop(serde_json::from_value(raw_log)?),
        "INBOUND_TCP" => handle_inbound_traffic(serde_json::from_value(raw_log)?),
        "OUTBOUND_TCP" => handle_outbound_traffic(serde_json::from_value(raw_log)?),
        _ => bail!("invalid sourcetype")
    };

    Ok(graph)
}

fn main() {

    handle_json_encoded_logs(
        move |raw_logs| {
            info!("Handling raw log");
            raw_logs.into_iter().map(handle_log).collect()
        }
    );

}
