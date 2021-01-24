use std::convert::TryFrom;

use grapl_graph_descriptions::{file::FileState,
                               graph_description::*,
                               node::NodeT,
                               process::ProcessState};
use serde::{Deserialize,
            Serialize};
use tracing::*;

#[derive(Clone, Debug, Hash, Serialize, Deserialize)]
pub struct ProcessStart {
    process_id: u64,
    parent_process_id: u64,
    name: String,
    hostname: String,
    arguments: String,
    timestamp: u64,
    exe: Option<String>,
}

impl TryFrom<ProcessStart> for Graph {
    type Error = String;

    fn try_from(process_start: ProcessStart) -> Result<Self, Self::Error> {
        let mut graph = Graph::new(process_start.timestamp);

        let asset = AssetBuilder::default()
            .asset_id(process_start.hostname.clone())
            .hostname(process_start.hostname.clone())
            .build()?;

        let parent = ProcessBuilder::default()
            .hostname(process_start.hostname.clone())
            .state(ProcessState::Existing)
            .process_id(process_start.parent_process_id)
            .last_seen_timestamp(process_start.timestamp)
            .build()?;

        let child = ProcessBuilder::default()
            .hostname(process_start.hostname.clone())
            .process_name(process_start.name)
            .state(ProcessState::Created)
            .process_id(process_start.process_id)
            .created_timestamp(process_start.timestamp)
            .build()?;

        if let Some(exe_path) = process_start.exe {
            let child_exe = FileBuilder::default()
                .hostname(process_start.hostname)
                .state(FileState::Existing)
                .last_seen_timestamp(process_start.timestamp)
                .file_path(exe_path)
                .build()?;

            graph.add_edge(
                "bin_file",
                child.node_key.clone(),
                child_exe.node_key.clone(),
            );

            graph.add_edge(
                "files_on_asset",
                asset.node_key.clone(),
                child_exe.node_key.clone(),
            );

            info!("child_exe: {}", child_exe.clone().into_json());
            graph.add_node(child_exe);
        }

        graph.add_edge(
            "asset_processes",
            asset.node_key.clone(),
            parent.node_key.clone(),
        );

        graph.add_edge(
            "asset_processes",
            asset.node_key.clone(),
            child.node_key.clone(),
        );

        graph.add_edge("children", parent.node_key.clone(), child.node_key.clone());

        graph.add_node(asset);
        graph.add_node(parent);
        graph.add_node(child);

        Ok(graph)
    }
}
