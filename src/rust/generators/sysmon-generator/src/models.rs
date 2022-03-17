use chrono::{
    DateTime,
    Utc,
};
use rust_proto::graph_descriptions::*;
use sysmon_parser::{
    EventData,
    SysmonEvent,
};

use crate::error::{
    Result,
    SysmonGeneratorError,
};

mod file;
mod network;
mod process;

pub(crate) fn generate_graph_from_event(
    sysmon_event: &SysmonEvent,
) -> Result<Option<GraphDescription>> {
    let graph = match &sysmon_event.event_data {
        EventData::FileCreate(event_data) => {
            let graph = file::generate_file_create_subgraph(&sysmon_event.system, event_data)?;

            Some(graph)
        }
        EventData::ProcessCreate(event_data) => {
            let graph =
                process::generate_process_create_subgraph(&sysmon_event.system, event_data)?;

            Some(graph)
        }
        EventData::NetworkConnect(event_data) => {
            if event_data.initiated {
                let graph = network::generate_outbound_connection_subgraph(
                    &sysmon_event.system,
                    event_data,
                )?;

                Some(graph)
            } else {
                // TODO(inickles): fix graph model for networking and support this
                tracing::warn!("found inbound connection, which is not currenlty supported.");

                None
            }
        }
        // We do not expect to handle all Sysmon event types
        _ => None,
    };

    Ok(graph)
}

/// Returns the provided file path with the Windows Zone Identifier removed if present.
///
/// When files are downloaded via a browser (e.g. Internet Explorer), an alternative data stream (ADS) may be created
/// to store additional meta-data about the file. These ADS files are suffixed with `:Zone.Identifier`.
///
/// The purpose of these ADS files is to provide context to users and Windows Policy controls to keep users safe.
///
/// For example, Microsoft Word may open potentially unsafe `.docx` files marked with `:Zone.Identifier`
/// in Protected View if the URL security zone is not well trusted.
///
/// ["What are Zone Identifier Files?"](https://stackoverflow.com/questions/4496697/what-are-zone-identifier-files)
fn strip_file_zone_identifier(path: &str) -> &str {
    if path.ends_with(":Zone.Identifier") {
        &path[0..path.len() - ":Zone.Identifier".len()]
    } else {
        path
    }
}

// TODO(inickles): delete this, do not strip full path and update analyzers accordingly.
/// Gets the name of the process given a path to the executable.
fn get_image_name(image_path: &str) -> String {
    image_path
        .split('\\')
        .last()
        .map(|name| name.replace("- ", "").replace('\\', ""))
        .unwrap_or(image_path.to_string())
}

/// Converts a Sysmon UTC string to UNIX Epoch time
///
/// If the provided string is not parseable as a UTC timestamp, an error is returned.
pub(crate) fn utc_to_epoch(utc: &DateTime<Utc>) -> Result<u64> {
    let ts = utc.timestamp_millis();

    if ts < 0 {
        return Err(SysmonGeneratorError::NegativeEventTime(ts));
    }

    if ts < 1_000_000_000_000 {
        Ok(ts as u64 * 1000)
    } else {
        Ok(ts as u64)
    }
}
