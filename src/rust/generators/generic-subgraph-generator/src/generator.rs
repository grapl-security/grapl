use std::convert::TryFrom;

use async_trait::async_trait;
use grapl_graph_descriptions::graph_description::*;
use sqs_executor::{
    cache::{Cache, CacheResponse},
    errors::{CheckedError, Recoverable},
    event_handler::{CompletedEvents, EventHandler},
    event_status::EventStatus,
};
use tracing::*;

use crate::models::GenericEvent;

#[derive(thiserror::Error, Debug)]
pub enum GenericSubgraphGeneratorError {
    #[error("Unexpected")]
    Unexpected(String),
}

impl CheckedError for GenericSubgraphGeneratorError {
    fn error_type(&self) -> Recoverable {
        Recoverable::Transient
    }
}

/// Supports a generic serialization format for incoming logs. This allows the use of any log source
/// as long as it is preprocessed to use Grapl's generic serialization format.
///
/// Grapl's generic generator expects ZStandard compressed JSON logs. The log types (and the required information)
/// can be found in the [GenericEvent] enum definition. Each type specified is supported but does required that
/// a `"eventname"` field is appended to the object with a value matching the string specified on the
/// variant.
///
/// e.g. The following is a valid [ProcessStart](../models/process/start/struct.ProcessStart.html) event:
///
/// ```
/// {
///   "eventname": "PROCESS_START",
///   "process_id": 2,
///   "parent_process_id": 1,
///   "name": "example.exe",
///   "hostname": "EXAMPLE",
///   "arguments": "-c 123",
///   "exe": "C:\\Users\\test_user\\AppData\\Local\\Temp\\example.exe",
///   "timestamp": 123
/// }
/// ```
///
/// Keep in mind that this generator expects the logs to have been compressed with ZStandard before processing.
#[derive(Clone)]
pub struct GenericSubgraphGenerator<C>
where
    C: Cache + Clone + Send + Sync + 'static,
{
    cache: C,
}

impl<C> GenericSubgraphGenerator<C>
where
    C: Cache + Clone + Send + Sync + 'static,
{
    pub fn new(cache: C) -> Self {
        Self { cache }
    }

    /// Takes the incoming generic events and tries to convert them into a merged subgraph
    ///
    /// For each log:
    /// * Try to convert to a GenericEvent
    /// * Generate subgraph from event
    /// * Merge into Graph object
    ///
    /// Returns: A Graph, identities processed, and an optional report indicating if any errors occurred during processing
    pub(crate) async fn convert_events_to_subgraph(
        &mut self,
        events: Vec<GenericEvent>,
        completed: &mut CompletedEvents,
    ) -> Result<Graph, Result<(Graph, GenericSubgraphGeneratorError), GenericSubgraphGeneratorError>>
    {
        let mut final_subgraph = Graph::new(0);
        let mut failed: Option<eyre::Report> = None;

        for event in events {
            let identity = event.clone();

            if let Ok(CacheResponse::Hit) = self.cache.get(identity.clone()).await {
                // If this was a hit, skip over the event because we've already processed it
                continue;
            }

            let subgraph = match Graph::try_from(event) {
                Ok(subgraph) => subgraph,
                Err(e) => {
                    error!("Failed to generate subgraph with: {}", e);
                    failed = Some(eyre::Report::msg(e));
                    continue;
                }
            };

            completed.add_identity(identity, EventStatus::Success);
            final_subgraph.merge(&subgraph);
        }

        match failed {
            Some(e) if final_subgraph.is_empty() => Err(Err(
                GenericSubgraphGeneratorError::Unexpected(e.to_string()),
            )),
            Some(e) => Err(Ok((
                final_subgraph,
                GenericSubgraphGeneratorError::Unexpected(e.to_string()),
            ))),
            None => Ok(final_subgraph),
        }
    }
}

#[async_trait]
impl<C> EventHandler for GenericSubgraphGenerator<C>
where
    C: Cache + Clone + Send + Sync + 'static,
{
    type InputEvent = Vec<GenericEvent>;
    type OutputEvent = Graph;
    type Error = GenericSubgraphGeneratorError;

    #[tracing::instrument(skip(self, events, completed))]
    async fn handle_event(
        &mut self,
        events: Vec<GenericEvent>,
        completed: &mut CompletedEvents,
    ) -> Result<Self::OutputEvent, Result<(Self::OutputEvent, Self::Error), Self::Error>> {
        self.convert_events_to_subgraph(events, completed).await
    }
}
