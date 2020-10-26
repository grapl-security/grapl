use super::from_str;
use crate::parsers::{OSQueryResponse, PartiallyDeserializedOSQueryLog};
use grapl_graph_descriptions::file::FileState;
use grapl_graph_descriptions::graph_description::*;
use grapl_graph_descriptions::node::NodeT;
use grapl_graph_descriptions::process::ProcessState;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer, Serialize};
use std::convert::TryFrom;
use std::fmt::Display;
use std::str::FromStr;

/// See https://osquery.io/schema/4.5.0/#processes
#[derive(Serialize, Deserialize)]
pub(crate) struct OSQueryProcessQuery {
    #[serde(deserialize_with = "from_str")]
    pid: u64,
    name: Option<String>,
    path: String,
    cmdline: String,
    #[serde(deserialize_with = "from_str")]
    parent: i64,
    #[serde(deserialize_with = "from_str")]
    time: i64,
}

impl PartiallyDeserializedOSQueryLog {
    pub(crate) fn to_graph_from_grapl_processes(self) -> Result<Graph, failure::Error> {
        OSQueryResponse::<OSQueryProcessQuery>::try_from(self)
            .map(|response| Graph::try_from(response))?
    }
}

impl TryFrom<OSQueryResponse<OSQueryProcessQuery>> for Graph {
    type Error = failure::Error;

    fn try_from(process_event: OSQueryResponse<OSQueryProcessQuery>) -> Result<Self, Self::Error> {
        let mut graph = Graph::new(process_event.unix_time);

        // this field can be -1 in cases of error
        // https://osquery.io/schema/4.5.1/#processes
        let process_start_time = if process_event.columns.time == -1 {
            process_event.unix_time
        } else {
            process_event.columns.time as u64
        };

        let asset = AssetBuilder::default()
            .asset_id(process_event.host_identifier.clone())
            .hostname(process_event.host_identifier.clone())
            .build()
            .map_err(failure::err_msg)?;

        let child = ProcessBuilder::default()
            .asset_id(process_event.host_identifier.clone())
            .hostname(process_event.host_identifier.clone())
            .state(ProcessState::Created)
            .created_timestamp(process_start_time)
            .process_name(process_event.columns.name.clone().unwrap_or("".to_string()))
            .process_id(process_event.columns.pid)
            .build()
            .map_err(failure::err_msg)?;

        if !process_event.columns.path.is_empty() {
            let child_exe = FileBuilder::default()
                .asset_id(process_event.host_identifier.clone())
                .file_path(process_event.columns.path.clone())
                .state(FileState::Existing)
                .last_seen_timestamp(process_start_time)
                .build()
                .map_err(failure::err_msg)?;

            graph.add_edge(
                "bin_file",
                child.clone_node_key(),
                child_exe.clone_node_key(),
            );

            graph.add_edge(
                "files_on_asset",
                asset.clone_node_key(),
                child_exe.clone_node_key(),
            );

            graph.add_node(child_exe);
        }

        // OSQuery can record -1 for ppid if a parent is not able to be determined
        // https://osquery.io/schema/4.5.1/#process_events
        if process_event.columns.parent >= 0 {
            let parent_process = ProcessBuilder::default()
                .asset_id(process_event.host_identifier.clone())
                .hostname(process_event.host_identifier.clone())
                .state(ProcessState::Existing)
                .process_id(process_event.columns.parent as u64)
                .last_seen_timestamp(process_start_time)
                .build()
                .map_err(failure::err_msg)?;

            graph.add_edge(
                "children",
                parent_process.clone_node_key(),
                child.clone_node_key(),
            );

            graph.add_edge(
                "asset_processes",
                asset.clone_node_key(),
                parent_process.clone_node_key(),
            );

            graph.add_node(parent_process);
        }

        graph.add_edge(
            "asset_processes",
            asset.clone_node_key(),
            child.clone_node_key(),
        );

        graph.add_node(child);
        graph.add_node(asset);

        Ok(graph)
    }
}
