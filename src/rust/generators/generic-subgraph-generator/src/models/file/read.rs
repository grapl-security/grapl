use grapl_graph_descriptions::file::FileState;
use grapl_graph_descriptions::graph_description::*;
use grapl_graph_descriptions::node::NodeT;
use grapl_graph_descriptions::process::ProcessState;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

#[derive(Clone, Debug, Hash, Serialize, Deserialize)]
pub struct FileRead {
    reader_process_id: u64,
    reader_process_name: Option<String>,
    path: String,
    hostname: String,
    timestamp: u64,
}

impl TryFrom<FileRead> for Graph {
    type Error = String;

    fn try_from(file_read: FileRead) -> Result<Self, Self::Error> {
        let deleter = ProcessBuilder::default()
            .process_name(file_read.reader_process_name.unwrap_or_default())
            .hostname(file_read.hostname.clone())
            .state(ProcessState::Existing)
            .process_id(file_read.reader_process_id)
            .last_seen_timestamp(file_read.timestamp)
            .build()?;

        let file = FileBuilder::default()
            .hostname(file_read.hostname)
            .state(FileState::Existing)
            .last_seen_timestamp(file_read.timestamp)
            .file_path(file_read.path)
            .build()?;

        let mut graph = Graph::new(file_read.timestamp);

        graph.add_edge(
            "read_files",
            deleter.clone_node_key(),
            file.clone_node_key(),
        );
        graph.add_node(deleter);
        graph.add_node(file);

        Ok(graph)
    }
}
