use serde::{Serialize, Deserialize, Deserializer};
use std::convert::TryFrom;
use crate::parsers::{PartiallyDeserializedOSQueryLog, OSQueryResponse};
use serde::de::DeserializeOwned;
use grapl_graph_descriptions::graph_description::*;
use grapl_graph_descriptions::file::FileState;
use grapl_graph_descriptions::node::NodeT;
use grapl_graph_descriptions::process::ProcessState;
use std::str::FromStr;
use std::fmt::Display;
use super::from_str;

/// See https://osquery.io/schema/4.5.0/#processes
#[derive(Serialize, Deserialize)]
pub(crate) struct OSQueryFileQuery {
    target_path: String,
    action: OSQueryFileAction,
    #[serde(deserialize_with = "from_str")]
    inode: u64,
    md5: String,
    sha1: String,
    sha256: String,
    #[serde(deserialize_with = "from_str")]
    size: u64,
    #[serde(deserialize_with = "from_str")]
    time: u64

}

#[derive(Serialize, Deserialize)]
pub(crate) enum OSQueryFileAction {
    ACCESSED,
    ATTRIBUTES_MODIFIED,
    UPDATED,
    CREATED,
    DELETED,
    MOVED_FROM,
    MOVED_TO,
    OPENED
}

impl PartiallyDeserializedOSQueryLog {
    pub(crate) fn to_graph_from_grapl_files(self) -> Result<Graph, failure::Error> {
        OSQueryResponse::<OSQueryFileQuery>::try_from(self)
            .map(|response| Graph::try_from(response))?
    }
}

impl TryFrom<OSQueryResponse<OSQueryFileQuery>> for Graph {
    type Error = failure::Error;

    fn try_from(file_event: OSQueryResponse<OSQueryFileQuery>) -> Result<Self, Self::Error> {
        let mut graph = Graph::new(file_event.unix_time);

        let asset = AssetBuilder::default()
            .asset_id(file_event.host_identifier.clone())
            .hostname(file_event.host_identifier.clone())
            .build()
            .map_err(failure::err_msg)?;

        let mut subject_file_builder = FileBuilder::default();
        subject_file_builder
            .asset_id(file_event.host_identifier.clone())
            .hostname(file_event.host_identifier.clone())
            .file_path(file_event.columns.target_path.clone())
            .md5_hash(file_event.columns.md5.clone())
            .sha1_hash(file_event.columns.sha1.clone())
            .sha256_hash(file_event.columns.sha256.clone())
            .md5_hash(file_event.columns.md5.clone())
            .file_inode(file_event.columns.inode.clone());

        /*
            Technically this might not be 100% correct but the moved_to and moved_from events
            seem like they could easily be represented by using create/deletes.
         */
        match &file_event.columns.action {
            OSQueryFileAction::CREATED | OSQueryFileAction::MOVED_FROM => {
                subject_file_builder
                    .state(FileState::Created)
                    .created_timestamp(file_event.columns.time)
            },
            OSQueryFileAction::DELETED | OSQueryFileAction::MOVED_TO => {
                subject_file_builder
                    .state(FileState::Deleted)
                    .deleted_timestamp(file_event.columns.time)
            },
            _ => {
                subject_file_builder
                    .state(FileState::Existing)
                    .last_seen_timestamp(file_event.columns.time)
            }
        };

        let subject_file = subject_file_builder.build().map_err(failure::err_msg)?;

        graph.add_edge(
            "files_on_asset",
            asset.clone_node_key(),
            subject_file.clone_node_key(),
        );

        graph.add_node(asset);
        graph.add_node(subject_file);

        Ok(graph)
    }
}