use std::collections::HashMap;

use crate::graph_description::{
    Edge,
    EdgeList,
    GeneratedSubgraphs,
    Graph,
    Node
};
use dgraph_tonic::{Client as DgraphClient, Mutate, Mutation as DgraphMutation, MutationResponse};
use dgraph_query_lib::query::{
    Query,
    QueryBuilder
};
use dgraph_query_lib::mutation::{
    Mutation,
    MutationBuilder
};

use log::{
    info,
    error
};

use crate::node::NodeT;
use dgraph_query_lib::upsert::{Upsert, UpsertBlock};
use futures_retry::{FutureRetry, RetryPolicy};
use std::time::Duration;
use std::sync::Arc;

impl Graph {
    pub fn new(timestamp: u64) -> Self {
        Graph {
            nodes: HashMap::new(),
            edges: HashMap::new(),
            timestamp,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty() && self.edges.is_empty()
    }

    pub fn merge(&mut self, other: &Graph) {
        self.edges.extend(other.edges.clone());

        for (node_key, other_node) in other.nodes.iter() {
            self.nodes
                .entry(node_key.clone())
                .and_modify(|node| {
                    node.merge(other_node);
                })
                .or_insert_with(|| other_node.clone());
        }
    }

    pub fn add_node<N>(&mut self, node: N)
    where
        N: Into<Node>,
    {
        let node = node.into();
        let key = node.clone_node_key();

        self.nodes.insert(key.to_string(), node);
        self.edges
            .entry(key)
            .or_insert_with(|| EdgeList { edges: vec![] });
    }

    pub fn with_node<N>(mut self, node: N) -> Graph
    where
        N: Into<Node>,
    {
        self.add_node(node);
        self
    }

    pub fn add_edge(
        &mut self,
        edge_name: impl Into<String>,
        from: impl Into<String>,
        to: impl Into<String>,
    ) {
        let from = from.into();
        let to = to.into();
        let edge_name = edge_name.into();
        let edge = Edge {
            from: from.clone(),
            to,
            edge_name,
        };

        self.edges
            .entry(from)
            .or_insert_with(|| EdgeList {
                edges: Vec::with_capacity(1),
            })
            .edges
            .push(edge);
    }

    pub async fn perform_upsert(&self, dgraph_client: Arc<DgraphClient>) {
        self.upsert_nodes(dgraph_client.clone()).await;
    }

    async fn upsert_nodes(&self, dgraph_client: Arc<DgraphClient>) {
        let (query_blocks, mutation_units): (Vec<_>, Vec<_>) = self.nodes.iter()
            .map(|(node_key, node)| node.generate_upsert_components())
            .unzip();

        let query = QueryBuilder::default()
            .query_blocks(query_blocks)
            .build()
            .unwrap();

        let mutation = MutationBuilder::default()
            .set(mutation_units)
            .build()
            .unwrap();

        let upsert = Upsert::new(query).upsert_block(UpsertBlock::new(mutation));

        let response = Self::enforce_transaction(dgraph_client, upsert).await;
    }

    async fn enforce_transaction(client: Arc<DgraphClient>, upsert: Upsert) -> MutationResponse {
        let dgraph_mutations: Vec<_> = upsert.mutations.iter()
            .map(|upsert_block| {
                let mut dgraph_mutation = DgraphMutation::new();

                if let Some(condition) = &upsert_block.cond {
                    dgraph_mutation.set_cond(condition);
                }

                dgraph_mutation.set_set_json(&upsert_block.mutation.set);

                dgraph_mutation
            }).collect();

        let (response, attempts) = FutureRetry::new(|| {
            client.new_mutated_txn()
                .upsert_and_commit_now(upsert.query.clone(), dgraph_mutations.clone())
        }, Self::handle_upsert_err).await
            .expect("Surfaced an error despite retry strategy while performing an upsert.");

        info!("Performed upsert after {} attempts", attempts);

        response
    }

    fn handle_upsert_err(e: anyhow::Error) -> RetryPolicy<anyhow::Error> {
        error!("Failed to process upsert. Retrying in 3 seconds. Error: {}", e);

        RetryPolicy::WaitRetry(Duration::new(3, 0))
    }
}

impl GeneratedSubgraphs {
    pub fn new(subgraphs: Vec<Graph>) -> GeneratedSubgraphs {
        GeneratedSubgraphs { subgraphs }
    }
}
