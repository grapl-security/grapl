use async_trait::async_trait;

use grapl_graph_descriptions::graph_description::*;

use crate::models::GenericEvent;
use grapl_graph_descriptions::node::NodeT;
use sqs_lambda::cache::{Cache, CacheResponse, Cacheable};
use sqs_lambda::event_handler::{Completion, EventHandler, OutputEvent};
use std::convert::TryFrom;
use tracing::*;

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
    ) -> (Graph, Vec<impl Cacheable>, Option<eyre::Report>) {
        let mut final_subgraph = Graph::new(0);
        let mut failed: Option<eyre::Report> = None;
        let mut identities = Vec::with_capacity(events.len());

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

            identities.push(identity);
            final_subgraph.merge(&subgraph);
        }

        (final_subgraph, identities, failed)
    }
}

#[async_trait]
impl<C> EventHandler for GenericSubgraphGenerator<C>
where
    C: Cache + Clone + Send + Sync + 'static,
{
    type InputEvent = Vec<GenericEvent>;
    type OutputEvent = Graph;
    type Error = sqs_lambda::error::Error;

    #[tracing::instrument(skip(self, events))]
    async fn handle_event(
        &mut self,
        events: Vec<GenericEvent>,
    ) -> OutputEvent<Self::OutputEvent, Self::Error> {
        let (subgraph, processed_identities, error_report) =
            self.convert_events_to_subgraph(events).await;

        // if an error occurred while converting generic events to a subgraph, we should record it
        let mut completed_event = if let Some(event_error) = error_report {
            OutputEvent::new(Completion::Partial((
                subgraph,
                sqs_lambda::error::Error::ProcessingError(event_error.to_string()),
            )))
        } else {
            OutputEvent::new(Completion::Total(subgraph))
        };

        processed_identities
            .into_iter()
            .for_each(|identity| completed_event.add_identity(identity));

        completed_event
    }
}
