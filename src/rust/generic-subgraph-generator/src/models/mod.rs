mod process;
mod file;
mod network;

use process::{ProcessStart, ProcessStop};
use serde::de::Error;
use std::convert::TryFrom;
use tracing::*;
use graph_descriptions::graph_description::*;
use crate::models::file::{FileCreate, FileDelete, FileRead, FileWrite};
use crate::models::network::{ProcessOutboundConnectionLog, ProcessInboundConnectionLog};
use crate::models::process::ProcessPortBindLog;

#[derive(Clone, Debug, Hash)]
pub enum GenericEvent {
    ProcessStart(ProcessStart),
    ProcessStop(ProcessStop),
    FileCreate(FileCreate),
    FileDelete(FileDelete),
    FileRead(FileRead),
    FileWrite(FileWrite),
    ProcessOutboundConnectionLog(ProcessOutboundConnectionLog),
    ProcessInboundConnectionLog(ProcessInboundConnectionLog),
    ProcessPortBindLog(ProcessPortBindLog),
}

impl GenericEvent {
    pub(crate) fn from_value(raw_log: serde_json::Value) -> Result<GenericEvent, serde_json::Error> {
        let eventname = match raw_log
            .get("eventname")
            .and_then(|eventname| eventname.as_str())
        {
            Some(eventname) => eventname.to_owned(),
            None => return Err(serde_json::Error::custom("missing event_type")),
        };

        info!("Parsing log of type: {}", eventname);
        match &eventname[..] {
            "PROCESS_START" => Ok(GenericEvent::ProcessStart(serde_json::from_value(raw_log)?)),
            "PROCESS_STOP" => Ok(GenericEvent::ProcessStop(serde_json::from_value(raw_log)?)),
            "FILE_CREATE" => Ok(GenericEvent::FileCreate(serde_json::from_value(raw_log)?)),
            "FILE_DELETE" => Ok(GenericEvent::FileDelete(serde_json::from_value(raw_log)?)),
            "FILE_READ" => Ok(GenericEvent::FileRead(serde_json::from_value(raw_log)?)),
            "FILE_WRITE" => Ok(GenericEvent::FileWrite(serde_json::from_value(raw_log)?)),
            "OUTBOUND_TCP" => Ok(GenericEvent::ProcessOutboundConnectionLog(
                serde_json::from_value(raw_log)?,
            )),
            "INBOUND_TCP" => Ok(GenericEvent::ProcessInboundConnectionLog(
                serde_json::from_value(raw_log)?,
            )),
            e => Err(serde_json::Error::custom(format!(
                "Invalid event type: {}",
                e
            ))),
        }
    }
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