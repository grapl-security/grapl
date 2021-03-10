use std::convert::TryFrom;

use endpoint_plugin::{AssetNode,
                      FileNode,
                      IAssetNode,
                      IFileNode,
                      IProcessNode,
                      ProcessNode};
use grapl_graph_descriptions::graph_description::*;
use serde::{Deserialize,
            Serialize};

use super::from_str;
use crate::parsers::{OSQueryAction,
                     OSQueryResponse,
                     PartiallyDeserializedOSQueryLog};

/// See https://osquery.io/schema/4.5.0/#processes
#[derive(Serialize, Deserialize)]
pub(crate) struct OSQueryProcessFileQuery {
    #[serde(deserialize_with = "from_str")]
    fd: u64,
    path: String,
    #[serde(deserialize_with = "from_str")]
    pid: u64,
}

impl PartiallyDeserializedOSQueryLog {
    pub(crate) fn to_graph_from_grapl_process_file(
        self,
    ) -> Result<GraphDescription, failure::Error> {
        OSQueryResponse::<OSQueryProcessFileQuery>::try_from(self)
            .map(|response| GraphDescription::try_from(response))?
    }
}

impl TryFrom<OSQueryResponse<OSQueryProcessFileQuery>> for GraphDescription {
    type Error = failure::Error;

    fn try_from(
        process_file_event: OSQueryResponse<OSQueryProcessFileQuery>,
    ) -> Result<Self, Self::Error> {
        let mut graph = GraphDescription::new();

        let mut asset = AssetNode::new(AssetNode::static_strategy());
        asset
            .with_asset_id(process_file_event.host_identifier.clone())
            .with_hostname(process_file_event.host_identifier.clone());

        let mut process = ProcessNode::new(ProcessNode::session_strategy());
        process
            .with_asset_id(process_file_event.host_identifier.clone())
            .with_last_seen_timestamp(process_file_event.unix_time)
            .with_process_id(process_file_event.columns.pid);

        let mut file = FileNode::new(FileNode::session_strategy());
        file.with_asset_id(process_file_event.host_identifier.clone())
            .with_file_path(process_file_event.columns.path.clone());

        // TODO: maybe we should set deleted time and created time for the file here?
        match process_file_event.action {
            OSQueryAction::ADDED => {
                file.with_created_timestamp(process_file_event.unix_time);

                graph.add_edge(
                    "created_files",
                    process.clone_node_key(),
                    file.clone_node_key(),
                );
            }
            OSQueryAction::REMOVED => {
                file.with_deleted_timestamp(process_file_event.unix_time);

                graph.add_edge(
                    "deleted_files",
                    process.clone_node_key(),
                    file.clone_node_key(),
                );
            }
            _ => {
                file.with_last_seen_timestamp(process_file_event.unix_time);
            }
        };

        graph.add_edge(
            "files_on_asset",
            asset.clone_node_key(),
            file.clone_node_key(),
        );

        graph.add_edge(
            "asset_processes",
            asset.clone_node_key(),
            process.clone_node_key(),
        );

        graph.add_node(asset);
        graph.add_node(file);
        graph.add_node(process);

        Ok(graph)
    }
}
