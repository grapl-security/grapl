use std::convert::TryFrom;

use grapl_graph_descriptions::{file::FileState,
                               graph_description::*,
                               node::NodeT,
                               process::ProcessState};
use serde::{Deserialize,
            Serialize};
use tracing::*;

#[derive(Clone, Debug, Hash, Serialize, Deserialize)]
pub struct FileCreate {
    creator_process_id: u64,
    creator_process_name: Option<String>,
    path: String,
    hostname: String,
    timestamp: u64,
}

impl TryFrom<FileCreate> for Graph {
    type Error = String;

    fn try_from(file_create: FileCreate) -> Result<Self, Self::Error> {
        let asset = AssetBuilder::default()
            .hostname(file_create.hostname.clone())
            .asset_id(file_create.hostname.clone())
            .build()?;

        let creator = ProcessBuilder::default()
            .hostname(file_create.hostname.clone())
            .process_name(file_create.creator_process_name.unwrap_or_default())
            .state(ProcessState::Existing)
            .process_id(file_create.creator_process_id)
            .last_seen_timestamp(file_create.timestamp)
            .build()?;

        let file = FileBuilder::default()
            .hostname(file_create.hostname)
            .state(FileState::Created)
            .created_timestamp(file_create.timestamp)
            .file_path(file_create.path)
            .build()?;

        info!("file {}", file.clone().into_json());

        let mut graph = Graph::new(file_create.timestamp);

        graph.add_edge(
            "created_files",
            creator.clone_node_key(),
            file.clone_node_key(),
        );

        graph.add_edge(
            "asset_processes",
            asset.clone_node_key(),
            creator.clone_node_key(),
        );

        graph.add_edge(
            "files_on_asset",
            asset.clone_node_key(),
            file.clone_node_key(),
        );

        graph.add_node(asset);
        graph.add_node(creator);
        graph.add_node(file);

        Ok(graph)
    }
}
