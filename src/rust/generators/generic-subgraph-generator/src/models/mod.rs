mod file;
mod network;
mod process;

use std::convert::TryFrom;

use grapl_graph_descriptions::graph_description::*;
use process::{ProcessStart,
              ProcessStop};
use serde::{Deserialize,
            Serialize};

use crate::models::{file::{FileCreate,
                           FileDelete,
                           FileRead,
                           FileWrite},
                    network::{ProcessInboundConnectionLog,
                              ProcessOutboundConnectionLog},
                    process::ProcessPortBindLog};

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
    type Error = String;

    fn try_from(generic_event: GenericEvent) -> Result<Self, Self::Error> {
        match generic_event {
            GenericEvent::ProcessStart(event) => Graph::try_from(event),
            GenericEvent::ProcessStop(event) => Graph::try_from(event),
            GenericEvent::FileCreate(event) => Graph::try_from(event),
            GenericEvent::FileDelete(event) => Graph::try_from(event),
            GenericEvent::FileRead(event) => Graph::try_from(event),
            GenericEvent::FileWrite(event) => Graph::try_from(event),
            GenericEvent::ProcessOutboundConnectionLog(event) => Graph::try_from(event),
            GenericEvent::ProcessInboundConnectionLog(event) => Graph::try_from(event),
            GenericEvent::ProcessPortBindLog(_event) => unimplemented!(),
        }
    }
}
