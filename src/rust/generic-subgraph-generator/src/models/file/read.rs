use serde::{Deserialize, Serialize};
use graph_descriptions::graph_description::*;
use graph_descriptions::process::ProcessState;
use graph_descriptions::file::FileState;
use graph_descriptions::node::NodeT;

#[derive(Clone, Debug, Hash, Serialize, Deserialize)]
pub struct FileRead {
    reader_process_id: u64,
    reader_process_name: Option<String>,
    path: String,
    hostname: String,
    timestamp: u64,
    eventname: String,
}

impl From<FileRead> for Graph {
    fn from(file_read: FileRead) -> Self {
        let deleter = ProcessBuilder::default()
            .process_name(file_read.reader_process_name.unwrap_or_default())
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

        graph.add_edge(
            "read_files",
            deleter.clone_node_key(),
            file.clone_node_key(),
        );
        graph.add_node(deleter);
        graph.add_node(file);

        graph
    }
}