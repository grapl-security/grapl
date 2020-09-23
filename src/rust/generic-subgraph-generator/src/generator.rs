
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use graph_descriptions::graph_description::*;
use graph_descriptions::process::ProcessState;
use graph_descriptions::file::FileState;
use graph_descriptions::node::NodeT;
use sqs_lambda::event_handler::{EventHandler, OutputEvent, Completion};
use sqs_lambda::cache::{Cache, Cacheable, CacheResponse};
use crate::models::GenericEvent;
use std::convert::TryFrom;
use tracing::*;

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
    async fn convert_events_to_subgraph(&mut self, events: Vec<serde_json::Value>) -> (Graph, Vec<impl Cacheable>, Option<eyre::Report>) {
        let mut final_subgraph = Graph::new(0);
        let mut failed: Option<eyre::Report> = None;
        let mut identities = Vec::with_capacity(events.len());

        for event in events {
            let event: GenericEvent = match GenericEvent::from_value(event) {
                Ok(event) => event,
                Err(e) => {
                    error!("Failed to generate subgraph with: {}", e);
                    failed = Some(e.into());
                    continue;
                }
            };

            let identity = event.clone();

            if let Ok(CacheResponse::Hit) = self.cache.get(identity.clone()).await {
                // If this was a hit, skip over the event because we've already processed it
                continue;
            }

            let subgraph = match Graph::try_from(event) {
                Ok(subgraph) => subgraph,
                Err(e) => {
                    error!("Failed to generate subgraph with: {}", e);
                    failed = Some(e);
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
    type InputEvent = Vec<serde_json::Value>;
    type OutputEvent = Graph;
    type Error = sqs_lambda::error::Error;

    #[tracing::instrument(skip(self, events))]
    async fn handle_event(
        &mut self,
        events: Vec<serde_json::Value>,
    ) -> OutputEvent<Self::OutputEvent, Self::Error> {
        let (subgraph, processed_identities, error_report) = self.convert_events_to_subgraph(events).await;

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