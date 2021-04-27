use std::convert::TryFrom;

use async_trait::async_trait;
use grapl_graph_descriptions::graph_description::*;
use log::*;
use sqs_executor::{cache::Cache,
                   errors::{CheckedError,
                            Recoverable},
                   event_handler::{CompletedEvents,
                                   EventHandler}};

use crate::{metrics::OSQuerySubgraphGeneratorMetrics,
            parsers::PartiallyDeserializedOSQueryLog};

#[derive(Clone)]
pub struct OSQuerySubgraphGenerator<C>
where
    C: Cache + Clone + Send + Sync + 'static,
{
    cache: C,
    metrics: OSQuerySubgraphGeneratorMetrics,
}

impl<C> OSQuerySubgraphGenerator<C>
where
    C: Cache + Clone + Send + Sync + 'static,
{
    pub fn new(cache: C, metrics: OSQuerySubgraphGeneratorMetrics) -> Self {
        Self { cache, metrics }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum OSQuerySubgraphGeneratorError {
    #[error("Unexpected")]
    Unexpected(failure::Error),
}

impl CheckedError for OSQuerySubgraphGeneratorError {
    fn error_type(&self) -> Recoverable {
        Recoverable::Transient
    }
}

#[async_trait]
impl<C> EventHandler for OSQuerySubgraphGenerator<C>
where
    C: Cache + Clone + Send + Sync + 'static,
{
    type InputEvent = Vec<PartiallyDeserializedOSQueryLog>;
    type OutputEvent = GraphDescription;
    type Error = OSQuerySubgraphGeneratorError;

    async fn handle_event(
        &mut self,
        input: Self::InputEvent,
        _completed: &mut CompletedEvents,
    ) -> Result<Self::OutputEvent, Result<(Self::OutputEvent, Self::Error), Self::Error>> {
        info!("Processing {} incoming OSQuery log events.", input.len());

        let (subgraphs, errors): (Vec<_>, Vec<_>) = input
            .into_iter()
            .map(|log| GraphDescription::try_from(log))
            .partition(|result| result.is_ok());

        for res in errors.iter().map(|e| e.as_ref().err()) {
            self.metrics.report_handle_event_success(&res);
        }
        let final_subgraph = subgraphs
            .into_iter()
            .filter_map(|subgraph| subgraph.ok())
            .fold(GraphDescription::new(), |mut current_graph, subgraph| {
                current_graph.merge(&subgraph);
                current_graph
            });

        let errors: Vec<failure::Error> =
            errors.into_iter().filter_map(|item| item.err()).collect();

        if errors.is_empty() {
            Ok(final_subgraph)
        } else {
            let sqs_executor_error = errors
                .into_iter()
                .map(|err| OSQuerySubgraphGeneratorError::Unexpected(err))
                .next()
                .unwrap();

            Err(Ok((final_subgraph, sqs_executor_error)))
        }
    }
}
