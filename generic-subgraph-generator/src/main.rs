#![feature(nll)]
#[macro_use]
extern crate failure;
#[macro_use] extern crate log;
#[macro_use] extern crate serde_derive;

extern crate graph_descriptions;
extern crate graph_generator_lib;
extern crate serde;
extern crate serde_json;

use failure::Error;
use graph_descriptions::*;
use graph_generator_lib::handle_json_encoded_logs;
use serde_json::Value;


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

    let parent = ProcessDescription::new(
        HostIdentifier::AssetId(process_start.asset_id.clone()),
        ProcessState::Existing,
        process_start.ppid,
        process_start.timestamp,
        vec![],
        vec![]
    );

    let child = ProcessDescription::new(
        HostIdentifier::AssetId(process_start.asset_id.clone()),
        ProcessState::Created,
        process_start.pid,
        process_start.timestamp,
        process_start.name.into_bytes(),
        vec![]
    );

    if let Some(exe_path) = process_start.exe {

        let child_exe = FileDescription::new(
            HostIdentifier::AssetId(process_start.asset_id),
            FileState::Existing,
            process_start.timestamp,
            exe_path.into_bytes(),
        );

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
    let terminated_process = ProcessDescription::new(
        HostIdentifier::AssetId(process_stop.asset_id),
        ProcessState::Terminated,
        process_stop.pid,
        process_stop.timestamp,
        process_stop.name.into_bytes(),
        vec![]
    );

    let mut graph = GraphDescription::new(
        process_stop.timestamp
    );
    graph.add_node(terminated_process);

    graph
}

fn handle_file_delete(file_delete: FileDelete) -> GraphDescription {
    let deleter = ProcessDescription::new(
        HostIdentifier::AssetId(file_delete.asset_id.clone()),
        ProcessState::Existing,
        file_delete.deleter_pid,
        file_delete.timestamp,
        vec![],
        vec![]
    );

    let file = FileDescription::new(
        HostIdentifier::AssetId(file_delete.asset_id),
        FileState::Deleted,
        file_delete.timestamp,
        file_delete.path.into_bytes(),
    );

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
    let creator = ProcessDescription::new(
        HostIdentifier::AssetId(file_creator.asset_id.clone()),
        ProcessState::Existing,
        file_creator.creator_pid,
        file_creator.timestamp,
        vec![],
        vec![]
    );

    let file = FileDescription::new(
        HostIdentifier::AssetId(file_creator.asset_id),
        FileState::Created,
        file_creator.timestamp,
        file_creator.path.into_bytes(),
    );

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
    let deleter = ProcessDescription::new(
        HostIdentifier::AssetId(file_write.asset_id.clone()),
        ProcessState::Existing,
        file_write.writer_pid,
        file_write.timestamp,
        vec![],
        vec![]
    );

    let file = FileDescription::new(
        HostIdentifier::AssetId(file_write.asset_id),
        FileState::Existing,
        file_write.timestamp,
        file_write.path.into_bytes(),
    );

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
    let deleter = ProcessDescription::new(
        HostIdentifier::AssetId(file_read.asset_id.clone()),
        ProcessState::Existing,
        file_read.reader_pid,
        file_read.timestamp,
        vec![],
        vec![]
    );

    let file = FileDescription::new(
        HostIdentifier::AssetId(file_read.asset_id),
        FileState::Existing,
        file_read.timestamp,
        file_read.path.into_bytes(),
    );

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
