use async_trait::async_trait;
use rust_proto::graph_descriptions::*;
use sqs_executor::{
    cache::Cache,
    event_handler::{
        CompletedEvents,
        EventHandler,
    },
    event_status::EventStatus,
};

use crate::{
    error::SysmonGeneratorError,
    metrics::SysmonGeneratorMetrics,
    models,
};

#[derive(Clone)]
pub struct SysmonGenerator<C>
where
    C: Cache + Clone + Send + Sync + 'static,
{
    cache: C,
    metrics: SysmonGeneratorMetrics,
}

impl<C> SysmonGenerator<C>
where
    C: Cache + Clone + Send + Sync + 'static,
{
    pub fn new(cache: C, metrics: SysmonGeneratorMetrics) -> Self {
        Self { cache, metrics }
    }
}

#[async_trait]
impl<C> EventHandler for SysmonGenerator<C>
where
    C: Cache + Clone + Send + Sync + 'static,
{
    type InputEvent = Vec<sysmon_parser::SysmonEvent<'static>>;
    type OutputEvent = GraphDescription;
    type Error = SysmonGeneratorError;

    #[tracing::instrument(skip(self, events, completed))]
    async fn handle_event(
        &mut self,
        events: Self::InputEvent,
        completed: &mut CompletedEvents,
    ) -> Result<Self::OutputEvent, Result<(Self::OutputEvent, Self::Error), Self::Error>> {
        tracing::info!(
            message = "Processing incoming events.",
            num_events = events.len()
        );

        // Skip events we've successfully processed and stored in the event cache.
        let events = self.cache.filter_cached(&events).await;

        let mut first_error: Option<SysmonGeneratorError> = None;

        let final_subgraph = events
            .iter()
            .filter_map(|event| {
                let result: Result<Option<GraphDescription>, _> =
                    models::generate_graph_from_event(event);

                self.metrics.report_subgraph_generation(&result);

                match result {
                    Ok(graph) => {
                        completed.add_identity(event, EventStatus::Success);

                        graph
                    }
                    Err(error) => {
                        completed.add_identity(event, EventStatus::Failure);

                        if first_error.is_none() {
                            first_error = Some(error);
                        }

                        None
                    }
                }
            })
            .fold(GraphDescription::new(), |mut current_graph, subgraph| {
                current_graph.merge(&subgraph);
                current_graph
            });

        tracing::info!(
            message = "Completed mapping subgraphs.",
            num_graphs = completed.len()
        );

        let final_result = match (first_error, final_subgraph.is_empty()) {
            (None, _) => Ok(final_subgraph),
            (Some(error), false) => Err(Ok((final_subgraph, error))),
            (Some(error), true) => Err(Err(error)),
        };

        self.metrics.report_handle_event_success(&final_result);

        final_result
    }
}
