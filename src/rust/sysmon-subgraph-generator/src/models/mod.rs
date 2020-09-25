use chrono::{NaiveDateTime, DateTime, Utc};
use failure::Error;
use log::*;
use failure::bail;
use sysmon::Event;
use graph_descriptions::graph_description::*;

mod process;
mod file;
mod network;

pub(crate) trait SysmonTryFrom<T>: Sized {
    type Error;

    fn try_from(instance: T) -> Result<Self, Self::Error>;
}

impl SysmonTryFrom<Event> for Graph {
    type Error = failure::Error;

    fn try_from(instance: Event) -> Result<Self, Self::Error> {
        match instance {
            Event::ProcessCreate(event) => {
                info!("Handling process create");

                let result = process::generate_process_create_subgraph(&event);

                if let Err(e) = &result {
                    warn!("Failed to process process start event: {}", e);
                }

                result
            }
            Event::FileCreate(event) => {
                info!("FileCreate");

                let result = file::generate_file_create_subgraph(&event);

                if let Err(e) = &result {
                    warn!("Failed to process file create event: {}", e);
                }

                result
            }
            // Event::InboundNetwork(event) => {
            //     info!("InboundNetwork");
            //
            //     let result = network::generate_inbound_connection_subgraph(&event);
            //
            //     if let Err(e) = &result {
            //         warn!("Failed to process inbound network event: {}", e);
            //     }
            //
            //     result
            // }
            Event::OutboundNetwork(event) => {
                info!("OutboundNetwork");

                let result = network::generate_outbound_connection_subgraph(&event);

                if let Err(e) = &result {
                    warn!("Failed to process outbound network event: {}", e);
                }

                result
            }
            unsupported_event => {
                let message = format!("Unsupported event_type: {:?}", unsupported_event);

                warn!("{}", message);

                Err(failure::err_msg(message))
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
fn get_image_name(image_path: &str) -> Option<String> {
    image_path
        .split('\\')
        .last()
        .map(|name| name.replace("- ", "").replace('\\', ""))
}

/// Converts a Sysmon UTC string to UNIX Epoch time
///
/// If the provided string is not parseable as a UTC timestamp, an error is returned.
pub fn utc_to_epoch(utc: &str) -> Result<u64, Error> {
    let dt = NaiveDateTime::parse_from_str(utc, "%Y-%m-%d %H:%M:%S%.3f")?;

    let dt: DateTime<Utc> = DateTime::from_utc(dt, Utc);
    let ts = dt.timestamp_millis();

    if ts < 0 {
        bail!("Timestamp is negative")
    }

    if ts < 1_000_000_000_000 {
        Ok(ts as u64 * 1000)
    } else {
        Ok(ts as u64)
    }
}