use async_trait::async_trait;

use grapl_graph_descriptions::graph_description::*;
use sqs_lambda::cache::{Cache, CacheResponse};
use crate::metrics::OSQuerySubgraphGeneratorMetrics;
use std::borrow::Cow;
use log::*;
use sqs_lambda::event_handler::{EventHandler, OutputEvent, Completion};
use crate::parsers::PartiallyDeserializedOSQueryLog;
use std::convert::TryFrom;


#[derive(Clone)]
pub(crate) struct OSQuerySubgraphGenerator<C>
where
    C: Cache + Clone + Send + Sync + 'static
{
    cache: C,
    metrics: OSQuerySubgraphGeneratorMetrics,
}

impl<C> OSQuerySubgraphGenerator<C>
where
    C: Cache + Clone + Send + Sync + 'static
{
    pub fn new(cache: C, metrics: OSQuerySubgraphGeneratorMetrics) -> Self {
        Self { cache, metrics }
    }
}

#[async_trait]
impl<C> EventHandler for OSQuerySubgraphGenerator<C>
where
    C: Cache + Clone + Send + Sync + 'static
{
    type InputEvent = Vec<PartiallyDeserializedOSQueryLog>;
    type OutputEvent = Graph;
    type Error = sqs_lambda::error::Error;

    async fn handle_event(&mut self, input: Self::InputEvent) -> OutputEvent<Self::OutputEvent, Self::Error> {
        info!("Processing {} incoming OSQuery log events.", input.len());

        let (subgraphs, errors): (Vec<Option<Graph>>, Vec<Option<failure::Error>>) = input.into_iter()
            .map(|log| {
                match Graph::try_from(log) {
                    Ok(graph) => (Some(graph), None),
                    Err(e) => {
                        warn!("Unable to convert partial OSQuery log into subgraph. {}", e);
                        (None, Some(e))
                    }
                }
            }).unzip();

        let final_subgraph = subgraphs.into_iter()
            .filter_map(|subgraph| subgraph)
            .fold(Graph::new(0), |mut current_graph, subgraph| {
                current_graph.merge(&subgraph);
                current_graph
            });

        let mut errors: Vec<failure::Error> = errors.into_iter()
            .filter_map(|item| item)
            .collect();

        if errors.is_empty() {
            OutputEvent::new(Completion::Total(final_subgraph))
        } else {
            let sqs_lambda_error = errors.pop()
                .map(|err| sqs_lambda::error::Error::ProcessingError(err.to_string()))
                .unwrap();

            OutputEvent::new(Completion::Partial((final_subgraph, sqs_lambda_error)))
        }
    }
}