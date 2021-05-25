#![allow(non_camel_case_types)]

use std::convert::TryFrom;

use endpoint_plugin::{
    AssetNode,
    FileNode,
    IAssetNode,
    IFileNode,
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
pub(crate) struct OSQueryFileQuery {
    target_path: String,
    action: OSQueryFileAction,
    inode: String,
    md5: String,
    sha1: String,
    sha256: String,
    size: String,
    #[serde(deserialize_with = "from_str")]
    time: u64,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum OSQueryFileAction {
    Accessed,
    AttributesModified,
    Updated,
    Created,
    Deleted,
    MovedFrom,
    MovedTo,
    Opened,
}

impl PartiallyDeserializedOSQueryLog {
    pub(crate) fn to_graph_from_grapl_files(self) -> Result<GraphDescription, failure::Error> {
        OSQueryResponse::<OSQueryFileQuery>::try_from(self)
            .map(|response| GraphDescription::try_from(response))?
    }
}

impl TryFrom<OSQueryResponse<OSQueryFileQuery>> for GraphDescription {
    type Error = failure::Error;

    fn try_from(file_event: OSQueryResponse<OSQueryFileQuery>) -> Result<Self, Self::Error> {
        let mut graph = GraphDescription::new();

        let mut asset = AssetNode::new(AssetNode::static_strategy());
        asset
            .with_asset_id(file_event.host_identifier.clone())
            .with_hostname(file_event.host_identifier.clone());

        let mut subject_file = FileNode::new(FileNode::session_strategy());
        subject_file
            .with_asset_id(file_event.host_identifier.clone())
            .with_file_path(file_event.columns.target_path.clone())
            .with_last_seen_timestamp(file_event.columns.time);

        /*
           Technically this might not be 100% correct but the moved_to and moved_from events
           seem like they could easily be represented by using create/deletes.
        */
        match &file_event.columns.action {
            OSQueryFileAction::Created | OSQueryFileAction::MovedFrom => {
                subject_file.with_created_timestamp(file_event.columns.time)
            }
            OSQueryFileAction::Deleted | OSQueryFileAction::MovedTo => {
                subject_file.with_deleted_timestamp(file_event.columns.time)
            }
            _ => subject_file.with_last_seen_timestamp(file_event.columns.time),
        };

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
