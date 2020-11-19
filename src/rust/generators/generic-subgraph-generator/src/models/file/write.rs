use grapl_graph_descriptions::file::FileState;
use grapl_graph_descriptions::graph_description::*;
use grapl_graph_descriptions::node::NodeT;
use grapl_graph_descriptions::process::ProcessState;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

#[derive(Clone, Debug, Hash, Serialize, Deserialize)]
pub struct FileWrite {
    writer_pid: u64,
    writer_process_name: Option<String>,
    path: String,
    hostname: String,
    timestamp: u64,
}

impl TryFrom<FileWrite> for Graph {
    type Error = String;

    fn try_from(file_write: FileWrite) -> Result<Self, Self::Error> {
        let asset = AssetBuilder::default()
            .hostname(file_write.hostname.clone())
            .asset_id(file_write.hostname.clone())
            .build()?;

        let writer = ProcessBuilder::default()
            .process_name(file_write.writer_process_name.unwrap_or_default())
            .hostname(file_write.hostname.clone())
            .state(ProcessState::Existing)
            .process_id(file_write.writer_pid)
            .last_seen_timestamp(file_write.timestamp)
            .build()?;

        let file = FileBuilder::default()
            .hostname(file_write.hostname)
            .state(FileState::Existing)
            .last_seen_timestamp(file_write.timestamp)
            .file_path(file_write.path)
            .build()?;

        let mut graph = Graph::new(file_write.timestamp);

        graph.add_edge(
            "wrote_files",
            writer.clone_node_key(),
            file.clone_node_key(),
        );

        graph.add_edge(
            "asset_processes",
            asset.clone_node_key(),
            writer.clone_node_key(),
        );

        graph.add_edge(
            "files_on_asset",
            asset.clone_node_key(),
            file.clone_node_key(),
        );

        graph.add_node(asset);
        graph.add_node(writer);
        graph.add_node(file);

        Ok(graph)
    }
}
