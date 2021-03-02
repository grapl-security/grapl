use std::{collections::HashMap,
          sync::Arc};

use dgraph_query_lib::{condition::{Condition,
                                   ConditionValue},
                       mutation::{MutationBuilder,
                                  MutationPredicateValue,
                                  MutationUID,
                                  MutationUnit},
                       predicate::{Field,
                                   Predicate},
                       query::QueryBuilder,
                       queryblock::{QueryBlockBuilder,
                                    QueryBlockType},
                       upsert::{Upsert,
                                UpsertBlock}};
use dgraph_tonic::{Client as DgraphClient,
                   Mutate,
                   Mutation as DgraphMutation,
                   MutationResponse};
use futures::StreamExt;
use futures_retry::{FutureRetry,
                    RetryPolicy};
use grapl_utils::iter_ext::GraplIterExt;
use log::{error,
          info,
          warn};

use crate::{graph_description::{Edge,
                                EdgeList,
                                GeneratedSubgraphs,
                                Graph,
                                Node},
            node::NodeT};

const DGRAPH_CONCURRENCY_UPSERTS: usize = 8;
// DGraph Live Loader uses a size of 1,000 elements and they claim this has relatively good performance
const DGRAPH_UPSERT_CHUNK_SIZE: usize = 1024;

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

    pub fn add_node_without_edges<N>(&mut self, node: N)
    where
        N: Into<Node>,
    {
        let node = node.into();
        let key = node.clone_node_key();

        self.nodes.insert(key.to_string(), node);
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
        self.upsert_edges(dgraph_client).await;
    }

    async fn upsert_nodes(&self, dgraph_client: Arc<DgraphClient>) {
        let node_upserts: Vec<_> = self
            .nodes
            .iter()
            .map(|(_, node)| node.generate_upsert_components())
            .collect();

        let _: Vec<MutationResponse> = futures::stream::iter(
            node_upserts
                .into_iter()
                .chunks_owned(DGRAPH_UPSERT_CHUNK_SIZE),
        )
        .map(|upsert_chunk| {
            let (query_blocks, mutation_units): (Vec<_>, Vec<_>) = upsert_chunk.into_iter().unzip();

            let query = QueryBuilder::default()
                .query_blocks(query_blocks)
                .build()
                .unwrap();

            let mutation = MutationBuilder::default()
                .set(mutation_units)
                .build()
                .unwrap();

            let upsert = Upsert::new(query).upsert_block(UpsertBlock::new(mutation));

            Self::enforce_transaction(dgraph_client.clone(), upsert)
        })
        .buffer_unordered(DGRAPH_CONCURRENCY_UPSERTS)
        .collect::<Vec<_>>()
        .await;
    }

    async fn upsert_edges(&self, dgraph_client: Arc<DgraphClient>) {
        let all_edges: Vec<_> = self
            .edges
            .iter()
            .flat_map(|(_, EdgeList { edges })| edges)
            .map(
                |Edge {
                     from,
                     to,
                     edge_name,
                 }| (from.clone(), to.clone(), edge_name.clone()),
            )
            .collect();

        futures::stream::iter(all_edges.into_iter().chunks_owned(DGRAPH_UPSERT_CHUNK_SIZE))
            .map(|items| {
                let mut node_key_to_variable_map = HashMap::<String, String>::new();
                let mut mutation_units = vec![];

                for (from, to, edge_name) in items {
                    let from_key_variable = node_key_to_variable_map
                        .entry(from.clone())
                        .or_insert(format!("nk_{}", rand::random::<u128>()))
                        .clone();

                    let to_key_variable = node_key_to_variable_map
                        .entry(to.clone())
                        .or_insert(format!("nk_{}", rand::random::<u128>()))
                        .clone();

                    let mutation_unit =
                        MutationUnit::new(MutationUID::variable(&from_key_variable)).predicate(
                            &edge_name,
                            MutationPredicateValue::Edges(vec![MutationUID::variable(
                                &to_key_variable,
                            )]),
                        );

                    mutation_units.push(mutation_unit);
                }

                let mut query_blocks = vec![];

                // now that all of the node keys have been used, we should generate the queries to grab them
                // and store the associated node uids in the variables we generated previously
                for (node_key, variable) in &node_key_to_variable_map {
                    let query_block = QueryBlockBuilder::default()
                        .query_type(QueryBlockType::Var)
                        .root_filter(Condition::EQ(
                            format!("node_key"),
                            ConditionValue::string(node_key),
                        ))
                        .predicates(vec![Predicate::ScalarVariable(
                            variable.to_string(),
                            Field::new("uid"),
                        )])
                        .first(1)
                        .build()
                        .unwrap();

                    query_blocks.push(query_block);
                }

                let query = QueryBuilder::default()
                    .query_blocks(query_blocks)
                    .build()
                    .unwrap();

                let mutation = MutationBuilder::default()
                    .set(mutation_units)
                    .build()
                    .unwrap();

                let upsert = Upsert::new(query).upsert_block(UpsertBlock::new(mutation));

                Self::enforce_transaction(dgraph_client.clone(), upsert)
            })
            .buffer_unordered(DGRAPH_CONCURRENCY_UPSERTS)
            .collect::<Vec<_>>()
            .await;
    }

    async fn enforce_transaction(client: Arc<DgraphClient>, upsert: Upsert) -> MutationResponse {
        let dgraph_mutations: Vec<_> = upsert
            .mutations
            .iter()
            .map(|upsert_block| {
                let mut dgraph_mutation = DgraphMutation::new();

                if let Some(condition) = &upsert_block.cond {
                    dgraph_mutation.set_cond(condition);
                }

                dgraph_mutation
                    .set_set_json(&upsert_block.mutation.set)
                    .unwrap_or_else(|e| error!("Failed to set json for mutation: {}", e));

                dgraph_mutation
            })
            .collect();

        let (response, attempts) = FutureRetry::new(
            || {
                client
                    .new_mutated_txn()
                    .upsert_and_commit_now(upsert.query.clone(), dgraph_mutations.clone())
            },
            Self::handle_upsert_err,
        )
        .await
        .expect("Surfaced an error despite retry strategy while performing an upsert.");

        info!("Performed upsert after {} attempts", attempts);

        response
    }

    fn handle_upsert_err(e: anyhow::Error) -> RetryPolicy<anyhow::Error> {
        // it's expected that this will fire occasionally so we'll just warn.
        // it's okay if this does fire as retrying is typically the correct solution (transaction conflict)
        warn!(
            "Failed to process upsert. Retrying immediately. Error that occurred: {}",
            e
        );
        RetryPolicy::Repeat
    }
}

impl GeneratedSubgraphs {
    pub fn new(subgraphs: Vec<Graph>) -> GeneratedSubgraphs {
        GeneratedSubgraphs { subgraphs }
    }
}
