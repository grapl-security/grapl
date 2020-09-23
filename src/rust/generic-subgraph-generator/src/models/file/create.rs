use graph_descriptions::file::FileState;
use graph_descriptions::graph_description::*;
use graph_descriptions::node::NodeT;
use graph_descriptions::process::ProcessState;
use serde::{Deserialize, Serialize};
use tracing::*;

#[derive(Clone, Debug, Hash, Serialize, Deserialize)]
pub struct FileCreate {
    creator_process_id: u64,
    creator_process_name: Option<String>,
    path: String,
    hostname: String,
    timestamp: u64,
}

impl From<FileCreate> for Graph {
    fn from(file_create: FileCreate) -> Self {
        let creator = ProcessBuilder::default()
            .hostname(file_create.hostname.clone())
            .process_name(file_create.creator_process_name.unwrap_or_default())
            .state(ProcessState::Existing)
            .process_id(file_create.creator_process_id)
            .last_seen_timestamp(file_create.timestamp)
            .build()
            .unwrap();

        let file = FileBuilder::default()
            .hostname(file_create.hostname)
            .state(FileState::Created)
            .created_timestamp(file_create.timestamp)
            .file_path(file_create.path)
            .build()
            .unwrap();

        info!("file {}", file.clone().into_json());

        let mut graph = Graph::new(file_create.timestamp);

        graph.add_edge(
            "created_files",
            creator.clone_node_key(),
            file.clone_node_key(),
        );
        graph.add_node(creator);
        graph.add_node(file);

        graph
    }
}
