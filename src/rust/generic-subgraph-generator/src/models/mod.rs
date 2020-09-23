mod file;
mod network;
mod process;

use serde::{Deserialize, Serialize};
use crate::models::file::{FileCreate, FileDelete, FileRead, FileWrite};
use crate::models::network::{ProcessInboundConnectionLog, ProcessOutboundConnectionLog};
use crate::models::process::ProcessPortBindLog;
use graph_descriptions::graph_description::*;
use process::{ProcessStart, ProcessStop};
use serde::de::Error;
use std::convert::TryFrom;
use tracing::*;

#[derive(Clone, Debug, Hash, Serialize, Deserialize)]
#[serde(tag = "eventname")]
pub enum GenericEvent {
    #[serde(rename = "PROCESS_START")]
    ProcessStart(ProcessStart),
    #[serde(rename = "PROCESS_STOP")]
    ProcessStop(ProcessStop),
    #[serde(rename = "FILE_CREATE")]
    FileCreate(FileCreate),
    #[serde(rename = "FILE_DELETE")]
    FileDelete(FileDelete),
    #[serde(rename = "FILE_READ")]
    FileRead(FileRead),
    #[serde(rename = "FILE_WRITE")]
    FileWrite(FileWrite),
    #[serde(rename = "OUTBOUND_TCP")]
    ProcessOutboundConnectionLog(ProcessOutboundConnectionLog),
    #[serde(rename = "INBOUND_TCP")]
    ProcessInboundConnectionLog(ProcessInboundConnectionLog),
    #[serde(rename = "PROCESS_PORT_BIND")]
    #[serde(skip)]
    ProcessPortBindLog(ProcessPortBindLog),
}

impl TryFrom<GenericEvent> for Graph {
    type Error = eyre::Report;

    fn try_from(generic_event: GenericEvent) -> Result<Self, Self::Error> {
        match generic_event {
            GenericEvent::ProcessStart(event) => Ok(Graph::from(event)),
            GenericEvent::ProcessStop(event) => Ok(Graph::from(event)),
            GenericEvent::FileCreate(event) => Ok(Graph::from(event)),
            GenericEvent::FileDelete(event) => Ok(Graph::from(event)),
            GenericEvent::FileRead(event) => Ok(Graph::from(event)),
            GenericEvent::FileWrite(event) => Ok(Graph::from(event)),
            GenericEvent::ProcessOutboundConnectionLog(event) => Ok(Graph::from(event)),
            GenericEvent::ProcessInboundConnectionLog(event) => Ok(Graph::from(event)),
            GenericEvent::ProcessPortBindLog(_event) => unimplemented!(),
        }
    }
}
