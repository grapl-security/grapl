use std::{collections::HashMap,
          convert::TryFrom};

use grapl_graph_descriptions::graph_description::*;
use regex::Regex;
use serde::{de::DeserializeOwned,
            Deserialize,
            Serialize};
use serde_json::Value;

mod grapl_pack;

#[derive(Serialize, Deserialize)]
struct OSQueryResponse<C> {
    #[serde(rename = "hostIdentifier")]
    host_identifier: String,
    #[serde(rename = "calendarTime")]
    calendar_time: String,
    #[serde(rename = "unixTime")]
    unix_time: u64,
    columns: C,
    action: OSQueryAction,
}

#[derive(Serialize, Deserialize)]
enum OSQueryAction {
    #[serde(rename = "added")]
    ADDED,
    #[serde(rename = "removed")]
    REMOVED,
    Other(String),
}

/// OSQuery logs should be deserialized into this struct first and then converted into a [Graph].
///
/// When converting this struct into a [Graph], it internally re-deserializes into a [OSQueryResponse]
/// object with pack-specific columnar data.
#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct PartiallyDeserializedOSQueryLog {
    pub(crate) name: String,
    #[serde(rename = "unixTime")]
    pub(crate) unix_time: u64,
    // holds other fields so we don't lose information by deserializing into PartialOSQueryResponse
    #[serde(flatten)]
    other_fields: HashMap<String, Value>,
}

impl TryFrom<PartiallyDeserializedOSQueryLog> for GraphDescription {
    type Error = failure::Error;

    /// Takes a [PartialOSQueryResponse], parses the pack and query name from the name field and
    /// attempts to deserialize the underlying log data into a Subgraph
    fn try_from(response: PartiallyDeserializedOSQueryLog) -> Result<Self, Self::Error> {
        let pack_and_query_name = response.name.clone();

        let pack_regex = Regex::new(r"pack_([^_]+)_(.+)").unwrap();
        let pack_match = pack_regex
            .captures(&pack_and_query_name)
            .ok_or(failure::err_msg(format!(
                "Failed to parse OSQuery log name field: {}",
                &response.name
            )))?;

        let pack_name = pack_match
            .get(1)
            .map(|m| m.as_str())
            .ok_or(failure::err_msg(format!(
                "Unable to parse pack name from OSQuery log name field: {}",
                &response.name
            )))?;

        let query_name = pack_match
            .get(2)
            .map(|m| m.as_str())
            .ok_or(failure::err_msg(format!(
                "Unable to parse query name from OSQuery log name field: {}",
                &response.name
            )))?;

        match (pack_name, query_name) {
            ("grapl", "processes") => response.to_graph_from_grapl_processes(),
            ("grapl", "files") => response.to_graph_from_grapl_files(),
            ("grapl", "process-files") => response.to_graph_from_grapl_process_file(),
            _ => Ok(GraphDescription::new()),
        }
    }
}

impl<T> TryFrom<PartiallyDeserializedOSQueryLog> for OSQueryResponse<T>
where
    T: DeserializeOwned,
{
    type Error = failure::Error;

    fn try_from(
        partial_query_response: PartiallyDeserializedOSQueryLog,
    ) -> Result<Self, Self::Error> {
        serde_json::to_value(partial_query_response)
            .map(|value| {
                serde_json::from_value(value).map_err(|err| failure::err_msg(err.to_string()))
            })
            .map_err(|err| failure::err_msg(err.to_string()))?
    }
}
