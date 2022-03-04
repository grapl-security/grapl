use chrono::{
    DateTime,
    Utc,
};
use rust_proto::graph_descriptions::*;
use sysmon_parser::SysmonEvent;

use crate::generator::SysmonGeneratorError;

mod file;
mod network;
mod process;

/// Because this crate doesn't own sysmon::Event nor rust_proto::graph_descriptions::GraphDescription
/// we need to create a new Trait to add a function to graph for Event.
///
/// Mimics TryFrom
pub(crate) trait SysmonTryFrom<T>: Sized {
    type Error;

    fn try_from(instance: T) -> Result<Self, Self::Error>;
}

impl SysmonTryFrom<&SysmonEvent<'_>> for GraphDescription {
    type Error = SysmonGeneratorError;

    #[tracing::instrument]
    fn try_from(sysmon_event: &SysmonEvent) -> Result<Self, Self::Error> {
        tracing::info!(event =? sysmon_event.system.event_id);

        use sysmon_parser::EventData;

        match &sysmon_event.event_data {
            EventData::FileCreate(event_data) => {
                let result = file::generate_file_create_subgraph(&sysmon_event.system, event_data);

                if let Err(e) = &result {
                    tracing::warn!(message="Failed to process event.", event_type =? sysmon_event.system.event_id, error=?e);
                }

                result
            }
            EventData::NetworkConnect(event_data) => {
                if event_data.initiated {
                    let result = network::generate_outbound_connection_subgraph(
                        &sysmon_event.system,
                        event_data,
                    );

                    if let Err(e) = &result {
                        tracing::warn!(message="Failed to process event.", event_type =? sysmon_event.system.event_id, error=?e);
                    }

                    result
                } else {
                    Err(SysmonGeneratorError::UnsupportedEventType(
                        "Inbound network events not supported, pending refactor.".to_string(),
                    ))
                }
            }
            EventData::ProcessCreate(event_data) => {
                let result =
                    process::generate_process_create_subgraph(&sysmon_event.system, event_data);

                if let Err(e) = &result {
                    tracing::warn!(message="Failed to process event.", event_type =? sysmon_event.system.event_id, error=?e);
                }

                result
            }
            _ => {
                tracing::warn!(
                    message = "Unsupported event_type",
                    event_type =? sysmon_event.system.event_id
                );

                Err(SysmonGeneratorError::UnsupportedEventType(format!(
                    "{:?}",
                    sysmon_event.system.event_id
                )))
            }
        }
    }
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
pub(crate) fn utc_to_epoch(utc: &DateTime<Utc>) -> Result<u64, SysmonGeneratorError> {
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
