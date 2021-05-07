use std::convert::TryFrom;

use endpoint_plugin::{
    AssetNode,
    FileNode,
    IAssetNode,
    IFileNode,
    IProcessNode,
    ProcessNode,
};
use grapl_graph_descriptions::graph_description::*;
use serde::{
    Deserialize,
    Serialize,
};

use super::from_str;
use crate::parsers::{
    OSQueryResponse,
    PartiallyDeserializedOSQueryLog,
};

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
    pub(crate) fn to_graph_from_grapl_processes(self) -> Result<GraphDescription, failure::Error> {
        OSQueryResponse::<OSQueryProcessQuery>::try_from(self)
            .map(|response| GraphDescription::try_from(response))?
    }
}

impl TryFrom<OSQueryResponse<OSQueryProcessQuery>> for GraphDescription {
    type Error = failure::Error;

    fn try_from(process_event: OSQueryResponse<OSQueryProcessQuery>) -> Result<Self, Self::Error> {
        let mut graph = GraphDescription::new();

        // this field can be -1 in cases of error
        // https://osquery.io/schema/4.5.1/#processes
        let process_start_time = if process_event.columns.time == -1 {
            process_event.unix_time
        } else {
            process_event.columns.time as u64
        };

        let mut asset = AssetNode::new(AssetNode::static_strategy());
        asset
            .with_asset_id(process_event.host_identifier.clone())
            .with_hostname(process_event.host_identifier.clone());

        let mut child = ProcessNode::new(ProcessNode::session_strategy());
        child
            .with_asset_id(process_event.host_identifier.clone())
            .with_created_timestamp(process_start_time)
            .with_last_seen_timestamp(process_start_time)
            .with_process_name(process_event.columns.name.clone().unwrap_or("".to_string()))
            .with_process_id(process_event.columns.pid);

        if !process_event.columns.path.is_empty() {
            let mut child_exe = FileNode::new(FileNode::session_strategy());
            child_exe
                .with_asset_id(process_event.host_identifier.clone())
                .with_file_path(process_event.columns.path.clone())
                .with_last_seen_timestamp(process_start_time);

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
            let mut parent_process = ProcessNode::new(ProcessNode::session_strategy());
            parent_process
                .with_asset_id(process_event.host_identifier.clone())
                .with_process_id(process_event.columns.parent as u64)
                .with_last_seen_timestamp(process_start_time);

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
