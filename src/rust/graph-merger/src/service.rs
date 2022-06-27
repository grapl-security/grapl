use std::{
    fmt::Debug,
    sync::Arc,
    time::{
        SystemTime,
        UNIX_EPOCH,
    },
};

use dgraph_tonic::Client as DgraphClient;
use rust_proto::graplinc::grapl::api::graph::v1beta1::{
    IdentifiedGraph,
    MergedGraph,
};

use crate::{
    reverse_resolver::ReverseEdgeResolver,
    upserter,
};

#[non_exhaustive]
#[derive(thiserror::Error, Debug)]
pub enum GraphMergerError {
    #[error("unexpected error")]
    Unexpected(String),

    #[error("error processing event {0}")]
    StreamProcessorError(#[from] kafka::StreamProcessorError),

    #[error("missing environment variable {0}")]
    MissingEnvironmentVariable(#[from] std::env::VarError),

    #[error("kafka configuration error {0}")]
    KafkaConfigurationError(#[from] kafka::ConfigurationError),

    #[error("error configuring tracing {0}")]
    TraceError(#[from] opentelemetry::trace::TraceError),

    #[error("anyhow error {0}")]
    AnyhowError(#[from] anyhow::Error),
}

impl From<GraphMergerError> for kafka::StreamProcessorError {
    fn from(graph_merger_error: GraphMergerError) -> Self {
        kafka::StreamProcessorError::EventHandlerError(graph_merger_error.to_string())
    }
}

impl From<&GraphMergerError> for kafka::StreamProcessorError {
    fn from(graph_merger_error: &GraphMergerError) -> Self {
        kafka::StreamProcessorError::EventHandlerError(graph_merger_error.to_string())
    }
}

#[derive(Clone)]
pub struct GraphMerger {
    mg_client: Arc<DgraphClient>,
    reverse_edge_resolver: ReverseEdgeResolver,
}

impl GraphMerger {
    pub fn new(mg_client: DgraphClient, reverse_edge_resolver: ReverseEdgeResolver) -> Self {
        Self {
            mg_client: Arc::new(mg_client),
            reverse_edge_resolver,
        }
    }

    #[tracing::instrument(skip(self, subgraph))]
    pub async fn handle_event(
        &mut self,
        subgraph: IdentifiedGraph,
    ) -> Result<MergedGraph, Result<(MergedGraph, GraphMergerError), GraphMergerError>> {
        if subgraph.is_empty() {
            tracing::warn!("Attempted to merge empty subgraph. Short circuiting.");
            return Ok(MergedGraph::default());
        }

        tracing::info!(
            message = "handling new subgraph",
            nodes =? subgraph.nodes.len(),
            edges =? subgraph.edges.len(),
        );

        let uncached_nodes = subgraph.nodes.into_iter().map(|(_, n)| n);
        let mut uncached_edges: Vec<_> =
            subgraph.edges.into_iter().flat_map(|e| e.1.edges).collect();
        let reverse = self
            .reverse_edge_resolver
            .resolve_reverse_edges(uncached_edges.clone())
            .await
            .map_err(Err)?;

        uncached_edges.extend_from_slice(&reverse[..]);

        let mut merged_graph = MergedGraph::new();
        let mut uncached_subgraph = IdentifiedGraph::new();

        for node in uncached_nodes {
            uncached_subgraph.add_node(node);
        }

        for edge in uncached_edges {
            uncached_subgraph.add_edge(edge.edge_name, edge.from_node_key, edge.to_node_key);
        }

        upserter::GraphMergeHelper {}
            .upsert_into(
                self.mg_client.clone(),
                &uncached_subgraph,
                &mut merged_graph,
            )
            .await;

        Ok(merged_graph)
    }
}

pub fn time_based_key_fn(_event: &[u8]) -> String {
    let cur_ms = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(n) => n.as_millis(),
        Err(_) => panic!("SystemTime before UNIX EPOCH!"),
    };

    let cur_day = cur_ms - (cur_ms % 86400);

    format!("{}/{}-{}", cur_day, cur_ms, uuid::Uuid::new_v4())
}
