use std::convert::TryFrom;

use grapl_graph_descriptions::{
    file::FileState, graph_description::*, node::NodeT, process::ProcessState,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Hash, Serialize, Deserialize)]
pub struct FileDelete {
    deleter_process_id: u64,
    deleter_process_name: Option<String>,
    path: String,
    hostname: String,
    timestamp: u64,
}

impl TryFrom<FileDelete> for Graph {
    type Error = String;

    fn try_from(file_delete: FileDelete) -> Result<Self, Self::Error> {
        let asset = AssetBuilder::default()
            .hostname(file_delete.hostname.clone())
            .asset_id(file_delete.hostname.clone())
            .build()?;

        let deleter = ProcessBuilder::default()
            .hostname(file_delete.hostname.clone())
            .state(ProcessState::Existing)
            .process_name(file_delete.deleter_process_name.unwrap_or_default())
            .process_id(file_delete.deleter_process_id)
            .last_seen_timestamp(file_delete.timestamp)
            .build()?;

        let file = FileBuilder::default()
            .hostname(file_delete.hostname)
            .state(FileState::Deleted)
            .deleted_timestamp(file_delete.timestamp)
            .file_path(file_delete.path)
            .build()?;

        let mut graph = Graph::new(file_delete.timestamp);

        graph.add_edge("deleted", deleter.clone_node_key(), file.clone_node_key());

        graph.add_edge(
            "asset_processes",
            asset.clone_node_key(),
            deleter.clone_node_key(),
        );

        graph.add_edge(
            "files_on_asset",
            asset.clone_node_key(),
            file.clone_node_key(),
        );

        graph.add_node(asset);
        graph.add_node(deleter);
        graph.add_node(file);

        Ok(graph)
    }
}
