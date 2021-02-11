use std::convert::TryFrom;

use grapl_graph_descriptions::{file::FileState,
                               graph_description::*,
                               node::NodeT,
                               process::ProcessState};
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
    pub(crate) fn to_graph_from_grapl_process_file(self) -> Result<Graph, failure::Error> {
        OSQueryResponse::<OSQueryProcessFileQuery>::try_from(self)
            .map(|response| Graph::try_from(response))?
    }
}

impl TryFrom<OSQueryResponse<OSQueryProcessFileQuery>> for Graph {
    type Error = failure::Error;

    fn try_from(
        process_file_event: OSQueryResponse<OSQueryProcessFileQuery>,
    ) -> Result<Self, Self::Error> {
        let mut graph = Graph::new(process_file_event.unix_time);

        let asset = AssetBuilder::default()
            .asset_id(process_file_event.host_identifier.clone())
            .hostname(process_file_event.host_identifier.clone())
            .build()
            .map_err(failure::err_msg)?;

        let process = ProcessBuilder::default()
            .asset_id(process_file_event.host_identifier.clone())
            .hostname(process_file_event.host_identifier.clone())
            .state(ProcessState::Existing)
            .last_seen_timestamp(process_file_event.unix_time)
            .process_id(process_file_event.columns.pid)
            .build()
            .map_err(failure::err_msg)?;

        let mut file_builder = FileBuilder::default();
        file_builder
            .asset_id(process_file_event.host_identifier.clone())
            .file_path(process_file_event.columns.path.clone());

        // TODO: maybe we should set deleted time and created time for the file here?
        let file = match process_file_event.action {
            OSQueryAction::ADDED => {
                let file = file_builder
                    .state(FileState::Created)
                    .created_timestamp(process_file_event.unix_time)
                    .build()
                    .map_err(failure::err_msg)?;

                graph.add_edge(
                    "created_files",
                    process.clone_node_key(),
                    file.clone_node_key(),
                );

                file
            }
            OSQueryAction::REMOVED => {
                let file = file_builder
                    .state(FileState::Deleted)
                    .deleted_timestamp(process_file_event.unix_time)
                    .build()
                    .map_err(failure::err_msg)?;

                graph.add_edge(
                    "deleted_files",
                    process.clone_node_key(),
                    file.clone_node_key(),
                );

                file
            }
            _ => file_builder
                .state(FileState::Deleted)
                .last_seen_timestamp(process_file_event.unix_time)
                .build()
                .map_err(failure::err_msg)?,
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
