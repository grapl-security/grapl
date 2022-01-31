use std::{
    fmt::Debug,
    sync::Arc,
    time::{
        SystemTime,
        UNIX_EPOCH,
    },
};

use async_trait::async_trait;
use dgraph_tonic::Client as DgraphClient;
use rust_proto::graph_descriptions::{
    IdentifiedGraph,
    MergedGraph,
};
use sqs_executor::{
    errors::{
        CheckedError,
        Recoverable,
    },
    event_handler::{
        CompletedEvents,
        EventHandler,
    },
};
use tracing::{
    error,
    info,
    warn,
};

use crate::{
    reverse_resolver::ReverseEdgeResolver,
    upserter,
};

#[derive(Clone)]
pub struct GraphMerger {
    mg_client: Arc<DgraphClient>,
    reverse_edge_resolver: ReverseEdgeResolver,
}

impl GraphMerger {
    pub fn new(mg_alphas: Vec<String>, reverse_edge_resolver: ReverseEdgeResolver) -> Self {
        let mg_client = DgraphClient::new(mg_alphas).expect("Failed to create dgraph client.");

        Self {
            mg_client: Arc::new(mg_client),
            reverse_edge_resolver,
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum GraphMergerError {
    #[error("UnexpectedError")]
    Unexpected(String),
}

impl CheckedError for GraphMergerError {
    fn error_type(&self) -> Recoverable {
        Recoverable::Transient
    }
}

#[async_trait]
impl EventHandler for GraphMerger {
    type InputEvent = IdentifiedGraph;
    type OutputEvent = MergedGraph;
    type Error = GraphMergerError;

    async fn handle_event(
        &mut self,
        subgraph: Self::InputEvent,
        _completed: &mut CompletedEvents,
    ) -> Result<Self::OutputEvent, Result<(Self::OutputEvent, Self::Error), Self::Error>> {
        if subgraph.is_empty() {
            warn!("Attempted to merge empty subgraph. Short circuiting.");
            return Ok(MergedGraph::default());
        }

        info!(
            message=
            "handling new subgraph",
            nodes=?subgraph.nodes.len(),
            edges=?subgraph.edges.len(),
        );

        let uncached_nodes = subgraph.nodes.into_iter().map(|(_, n)| n);
        let mut uncached_edges: Vec<_> = subgraph
            .edges
            .into_iter()
            .flat_map(|e| e.1.into_vec())
            .collect();
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
