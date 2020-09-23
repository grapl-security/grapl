use graph_descriptions::file::FileState;
use graph_descriptions::graph_description::*;
use graph_descriptions::node::NodeT;
use graph_descriptions::process::ProcessState;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Hash, Serialize, Deserialize)]
pub struct FileDelete {
    deleter_process_id: u64,
    deleter_process_name: Option<String>,
    path: String,
    hostname: String,
    timestamp: u64,
}

impl From<FileDelete> for Graph {
    fn from(file_delete: FileDelete) -> Self {
        let deleter = ProcessBuilder::default()
            .hostname(file_delete.hostname.clone())
            .state(ProcessState::Existing)
            .process_name(file_delete.deleter_process_name.unwrap_or_default())
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
}
