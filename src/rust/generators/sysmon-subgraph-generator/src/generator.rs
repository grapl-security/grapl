use std::time::Instant;

use async_trait::async_trait;
use grapl_graph_descriptions::graph_description::*;
use grapl_observe::log_time;
use log::*;
use sqs_executor::{cache::Cache,
                   errors::{CheckedError,
                            Recoverable},
                   event_handler::{CompletedEvents,
                                   EventHandler},
                   event_status::EventStatus};
use sysmon::Event;

use crate::{metrics::SysmonSubgraphGeneratorMetrics,
            models::SysmonTryFrom};

#[derive(thiserror::Error, Debug)]
pub enum SysmonGeneratorError {
    #[error("DeserializeError")]
    DeserializeError(failure::Error),
    #[error("NegativeEventTime")]
    NegativeEventTime(i64),
    #[error("TimeError")]
    TimeError(#[from] chrono::ParseError),
    #[error("Unsupported event type")]
    UnsupportedEventType(String),
}

impl CheckedError for SysmonGeneratorError {
    fn error_type(&self) -> Recoverable {
        match self {
            Self::DeserializeError(_) => Recoverable::Persistent,
            Self::NegativeEventTime(_) => Recoverable::Persistent,
            Self::TimeError(_) => Recoverable::Persistent,
            Self::UnsupportedEventType(_) => Recoverable::Persistent,
        }
    }
}

#[derive(Clone)]
pub(crate) struct SysmonSubgraphGenerator<C>
where
    C: Cache + Clone + Send + Sync + 'static,
{
    cache: C,
    metrics: SysmonSubgraphGeneratorMetrics,
}

impl<C> SysmonSubgraphGenerator<C>
where
    C: Cache + Clone + Send + Sync + 'static,
{
    pub fn new(cache: C, metrics: SysmonSubgraphGeneratorMetrics) -> Self {
        Self { cache, metrics }
    }

    /// Takes a vec of event Strings, parses them, and converts them into subgraphs
    async fn process_events(
        &mut self,
        events: Vec<String>,
        identities: &mut CompletedEvents,
    ) -> Result<
        GraphDescription,
        Result<(GraphDescription, SysmonGeneratorError), SysmonGeneratorError>,
    > {
        let mut last_error: Option<SysmonGeneratorError> = None;
        let mut final_subgraph = GraphDescription::new();

        // Skip events we've successfully processed and stored in the event cache.
        let events = self.cache.filter_cached(&events).await;

        for event in events {
            let event = match Event::from_str(&event) {
                Ok(event) => event,
                Err(e) => {
                    warn!("Failed to deserialize event: {}, {}", e, event);

                    last_error = Some(SysmonGeneratorError::DeserializeError(failure::err_msg(
                        format!("Failed: {}", e),
                    )));

                    continue;
                }
            };

            let graph = match GraphDescription::try_from(event.clone()) {
                Ok(subgraph) => subgraph,
                Err(SysmonGeneratorError::UnsupportedEventType(_s)) => continue,
                Err(new_error) => {
                    error!("GraphDescription::try_from failed with: {:?}", new_error);
                    // TODO: we should probably be recording each separate failure, but this is only going to save the last failure
                    match last_error {
                        // Save the last error, but do not overwrite a transient error with a persistent error.
                        // The awkwardness of this is being tracked in: https://github.com/grapl-security/issue-tracker/issues/269
                        Some(ref mut e) => {
                            if e.is_transient() {
                                last_error = Some(new_error);
                            }
                        }
                        None => {
                            last_error = Some(new_error);
                        }
                    }

                    continue;
                }
            };

            final_subgraph.merge(&graph);
            identities.add_identity(event, EventStatus::Success);
        }

        match (last_error, identities.identities.is_empty()) {
            (Some(last_failure), true) => Err(Err(last_failure)),
            (Some(last_failure), false) => Err(Ok((final_subgraph, last_failure))),
            (None, _) => Ok(final_subgraph),
        }
    }
}

#[async_trait]
impl<C> EventHandler for SysmonSubgraphGenerator<C>
where
    C: Cache + Clone + Send + Sync + 'static,
{
    type InputEvent = Vec<u8>;
    type OutputEvent = GraphDescription;
    type Error = SysmonGeneratorError;

    async fn handle_event(
        &mut self,
        events: Vec<u8>,
        completed: &mut CompletedEvents,
    ) -> Result<Self::OutputEvent, Result<(Self::OutputEvent, Self::Error), Self::Error>> {
        info!("Handling raw event");

        /*
           This iterator is taking a set of bytes of the logs, splitting the logs on newlines,
           converting the byte sequences to utf-8 strings, and then filtering on the following criteria:
               1. The line isn't empty
               2. The line is not `\n` (to prevent issues with multiple newline sequences)
               3. The line contains event with ID 1, 3, or 11

           The event ids 1, 3, and 11 correspond to Process Creation, Network Connection, and File Creation
           in that order.

           https://docs.microsoft.com/en-us/sysinternals/downloads/sysmon#events
        */
        let events: Vec<_> = log_time!(
            "event split",
            events
                .split(|i| &[*i][..] == &b"\n"[..])
                .map(String::from_utf8_lossy)
                .map(|s| s.to_string())
                .filter(|event| {
                    (!event.is_empty() && event != "\n")
                        && (event.contains(&"EventID>1<"[..])
                            || event.contains(&"EventID>3<"[..])
                            || event.contains(&"EventID>11<"[..]))
                })
                .collect()
        );

        info!("Handling {} events", events.len());

        let final_subgraph = self.process_events(events, completed).await;

        info!("Completed mapping {} subgraphs", completed.len());
        self.metrics.report_handle_event_success(&final_subgraph);

        final_subgraph
    }
}
