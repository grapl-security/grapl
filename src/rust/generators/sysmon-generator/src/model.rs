use rust_proto::graph_descriptions::*;
use sysmon_parser::{
    EventData,
    SysmonEvent,
};

use crate::error::Result;

mod events;
mod nodes;

#[tracing::instrument(err, skip(sysmon_event))]
pub(crate) fn generate_graph_from_event(
    sysmon_event: &SysmonEvent,
) -> Result<Option<GraphDescription>> {
    let graph = match &sysmon_event.event_data {
        EventData::FileCreate(event_data) => {
            let graph = events::file_created::generate_file_create_subgraph(
                &sysmon_event.system,
                event_data,
            );

            Some(graph)
        }
        EventData::ProcessCreate(event_data) => {
            let graph = events::process_created::generate_process_create_subgraph(
                &sysmon_event.system,
                event_data,
            );

            Some(graph)
        }
        EventData::NetworkConnect(event_data) => {
            if event_data.initiated {
                let graph = events::network_connection::generate_network_connection_subgraph(
                    &sysmon_event.system,
                    event_data,
                );

                Some(graph)
            } else {
                // TODO(inickles): fix graph model for networking and support this
                tracing::warn!("found inbound connection, which is not currenlty supported");

                None
            }
        }
        // We do not expect to handle all Sysmon event types
        _ => None,
    };

    tracing::debug!(
        message = "completed graph generation",
        node_count = graph.as_ref().map(|graph| graph.nodes.len()),
        edge_count = graph.as_ref().map(|graph| graph.edges.len()),
    );

    Ok(graph)
}
