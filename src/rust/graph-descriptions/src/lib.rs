// #![allow(unused_imports, unused_mut)]
#![allow(warnings)]
pub use crate::{graph_description::*, node_property::Property};
use dgraph_query_lib::mutation::Mutation;
use dgraph_query_lib::queryblock::QueryBlock;
use dgraph_query_lib::ToQueryString;
use dgraph_query_lib::{
    condition::{Condition, ConditionValue},
    mutation::{MutationBuilder, MutationPredicateValue, MutationUID, MutationUnit},
    predicate::{Field, Predicate},
    query::QueryBuilder,
    queryblock::{QueryBlockBuilder, QueryBlockType},
    upsert::{Upsert, UpsertBlock},
};

use dgraph_tonic::Query;

pub use node_property::Property::{
    DecrementOnlyIntProp as ProtoDecrementOnlyIntProp,
    DecrementOnlyUintProp as ProtoDecrementOnlyUintProp, ImmutableIntProp as ProtoImmutableIntProp,
    ImmutableStrProp as ProtoImmutableStrProp, ImmutableUintProp as ProtoImmutableUintProp,
    IncrementOnlyIntProp as ProtoIncrementOnlyIntProp,
    IncrementOnlyUintProp as ProtoIncrementOnlyUintProp,
};
use std::{collections::HashMap, sync::Arc};

use dgraph_tonic::{Client as DgraphClient, Mutate, Mutation as DgraphMutation, MutationResponse};
use futures::StreamExt;
use futures_retry::{FutureRetry, RetryPolicy};
use grapl_utils::iter_ext::GraplIterExt;

pub mod upsert;

const DGRAPH_CONCURRENCY_UPSERTS: usize = 8;
// DGraph Live Loader uses a size of 1,000 elements and they claim this has relatively good performance
const DGRAPH_UPSERT_CHUNK_SIZE: usize = 1024;

// A helper macro to generate `as_*` methods for the NodeProperty wrapper type, and its internall
// enumeration
macro_rules! impl_as {
    ($base_t:ty, $as_ident:ident, $r:pat, $p:ident, $to_t:ty, $variant:path) => {
        impl $base_t {
            pub fn $as_ident(&self) -> Option<$to_t> {
                match self.property {
                    Some($variant($r)) => Some($p),
                    _ => None,
                }
            }
        }
    };
    ($base_t:ty, $as_ident:ident, &$to_t:ty, $variant:path) => {
        impl_as! {$base_t, $as_ident, ref p, p, &$to_t, $variant}
    };
    ($base_t:ty, $as_ident:ident, $to_t:ty, $variant:path) => {
        impl_as! {$base_t, $as_ident,     p, p, $to_t, $variant}
    };
}

// A helper macro to generate `From` impl boilerplate.
macro_rules ! impl_from_for {
    ($into_t:ty, $field:tt, $from_t:ty) => {
        impl From<$from_t> for $into_t
        {
            fn from(p: $from_t) -> Self {
                let p = p.to_owned().into();
                Self {$field: p}
            }
        }
    };
    ($into_t:ty, $field:tt, $head:ty, $($tail:ty),*) => {
        impl_from_for!($into_t, $field, $head);
        impl_from_for!($into_t, $field, $($tail),*);
    };
    ($from_t:tt, $to_t:ty) => {
        impl From<$from_t> for $to_t {
            fn from($from_t (s): $from_t) -> Self {
                Self:: $from_t (s)
            }
        }
    };
}

pub mod graph_description {
    // TODO: Restructure the Rust modules to better reflect the new
    // Protobuf structure
    include!(concat!(
        env!("OUT_DIR"),
        "/graplinc.grapl.api.graph.v1beta1.rs"
    ));
}

impl GraphDescription {
    pub fn new() -> Self {
        Self {
            nodes: Default::default(),
            edges: Default::default(),
        }
    }

    pub fn add_node(&mut self, node: impl Into<NodeDescription>) {
        let node = node.into();
        match self.nodes.get_mut(&node.node_key) {
            Some(n) => n.merge(&node),
            None => {
                self.nodes.insert(node.clone_node_key(), node);
            }
        };
    }

    pub fn add_edge(
        &mut self,
        edge_name: impl Into<String>,
        from_node_key: impl Into<String>,
        to_node_key: impl Into<String>,
    ) {
        let from_node_key = from_node_key.into();
        let to_node_key = to_node_key.into();
        let edge_name = edge_name.into();

        assert_ne!(from_node_key, to_node_key);

        let edge = Edge {
            from_node_key: from_node_key.clone(),
            to_node_key,
            edge_name,
        };

        let edge_list: &mut Vec<Edge> = &mut self
            .edges
            .entry(from_node_key)
            .or_insert_with(|| EdgeList {
                edges: Vec::with_capacity(1),
            })
            .edges;
        edge_list.push(edge);
    }

    pub fn merge(&mut self, other: &Self) {
        for (node_key, other_node) in other.nodes.iter() {
            match self.nodes.get_mut(node_key) {
                Some(n) => n.merge(other_node),
                None => {
                    self.nodes.insert(node_key.clone(), other_node.clone());
                }
            };
        }

        for edge_list in other.edges.values() {
            for edge in edge_list.edges.iter() {
                self.add_edge(
                    edge.edge_name.clone(),
                    edge.from_node_key.clone(),
                    edge.to_node_key.clone(),
                );
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty() && self.edges.is_empty()
    }
}

impl IdentifiedGraph {
    pub fn new() -> Self {
        Self {
            nodes: Default::default(),
            edges: Default::default(),
        }
    }

    pub fn add_node(&mut self, node: IdentifiedNode) {
        match self.nodes.get_mut(&node.node_key) {
            Some(n) => n.merge(&node),
            None => {
                self.nodes.insert(node.clone_node_key(), node);
            }
        };
    }

    pub fn add_edge(
        &mut self,
        edge_name: impl Into<String>,
        from_node_key: impl Into<String>,
        to_node_key: impl Into<String>,
    ) {
        let from_node_key = from_node_key.into();
        let to_node_key = to_node_key.into();
        assert_ne!(from_node_key, to_node_key);

        let edge_name = edge_name.into();
        let edge = Edge {
            from_node_key: from_node_key.clone(),
            to_node_key,
            edge_name,
        };

        let edge_list: &mut Vec<Edge> = &mut self
            .edges
            .entry(from_node_key)
            .or_insert_with(|| EdgeList {
                edges: Vec::with_capacity(1),
            })
            .edges;
        edge_list.push(edge);
    }

    pub fn merge(&mut self, other: &Self) {
        for (node_key, other_node) in other.nodes.iter() {
            match self.nodes.get_mut(node_key) {
                Some(n) => n.merge(other_node),
                None => {
                    self.nodes.insert(node_key.clone(), other_node.clone());
                }
            };
        }

        for edge_list in other.edges.values() {
            for edge in edge_list.edges.iter() {
                self.add_edge(
                    edge.edge_name.clone(),
                    edge.from_node_key.clone(),
                    edge.to_node_key.clone(),
                );
            }
        }
    }

    pub async fn perform_upsert_into(
        &self,
        dgraph_client: Arc<DgraphClient>,
        merged_graph: &mut MergedGraph,
    ) {
        let node_key_map_to_uid = self.upsert_nodes(dgraph_client.clone(), merged_graph).await;
        self.upsert_edges(dgraph_client, node_key_map_to_uid).await;
    }

    async fn upsert_nodes(
        &self,
        dgraph_client: Arc<DgraphClient>,
        merged_graph: &mut MergedGraph,
    ) -> HashMap<String, u64> {
        let mut key_to_query_name = HashMap::new();
        let mut node_upserts = Vec::with_capacity(self.nodes.len());
        for (unique_id, node) in self.nodes.values().enumerate() {
            let (query, upserts) = upsert::build_upserts(
                unique_id as u128,
                &node.node_key,
                &node.node_type,
                &node.properties,
                &mut key_to_query_name,
            );
            node_upserts.push((query, upserts));
        }

        tracing::info!(message="Upserting nodes", count=node_upserts.len());
        let responses: Vec<dgraph_tonic::Response> = futures::stream::iter(
            node_upserts
                .into_iter()
                .chunks_owned(DGRAPH_UPSERT_CHUNK_SIZE),
        )
        .map(move |upsert_chunk| {
            let mut combined_query = String::new();
            let mut all_mutations = Vec::new();
            for (query_block, mutations) in upsert_chunk.iter() {
                combined_query.push_str(&query_block);
                all_mutations.extend_from_slice(mutations);
            }

            let combined_query = format!(
                r"
            {{
                    {}
            }}
            ",
                combined_query
            );

            tracing::debug!(message="Generated query for upsert", combined_query=?combined_query);

            let dgraph_client = dgraph_client.clone();
            Self::enforce_transaction(move || {
                let mut txn = dgraph_client.new_mutated_txn();
                txn.upsert_and_commit_now(combined_query.clone(), all_mutations.clone())
            })
        })
        .buffer_unordered(DGRAPH_CONCURRENCY_UPSERTS)
        .collect::<Vec<_>>()
        .await;

        let mut node_key_map_to_uid = HashMap::new();
        let mut uids = HashMap::new();
        for response in responses.iter() {
            let query_responses: serde_json::Value = match serde_json::from_slice(&response.json) {
                Ok(response) => response,
                Err(e) => {
                    tracing::error!(message="Failed to parse JSON response for upsert", error=?e);
                    continue;
                }
            };
            tracing::debug!(
                message="Received upsert response",
                query_response=?query_responses,
                uids=?response.uids,
            );
            uids.extend(response.uids.clone());
            extract_node_key_map_uid(&query_responses, &mut node_key_map_to_uid);
        }

        for node in self.nodes.values() {
            let IdentifiedNode {
                node_key,
                node_type,
                properties,
            } = node.to_owned();

            let uid = node_key_map_to_uid.get(&node_key)
                .map(|uid| *uid)
                .or_else(|| uid_from_uids(&node_key, &key_to_query_name, &uids ));
            let uid = match uid {
                Some(uid) => uid,
                None => {
                    tracing::error!(
                        message="Failed to retrieve uid associated with node_key",
                        node_key=?node_key,
                    );
                    continue;
                }
            };

            merged_graph.add_node(MergedNode {
                uid,
                node_key,
                node_type,
                properties,
            });
        }
        node_key_map_to_uid
    }

    async fn upsert_edges(
        &self,
        dgraph_client: Arc<DgraphClient>,
        mut node_key_to_uid: HashMap<String, u64>,
    ) {
        let all_edges: Vec<_> = self
            .edges
            .iter()
            .flat_map(|(_, EdgeList {edges})| edges)
            .map(
                |Edge {
                     from_node_key,
                     to_node_key,
                     edge_name,
                 }| {
                    (
                        from_node_key.clone(),
                        to_node_key.clone(),
                        edge_name.clone(),
                    )
                },
            )
            .collect();

        let mut unresolved = vec![];
        for (from_node_key, to_node_key, edge_name) in all_edges.iter() {
            if !node_key_to_uid.contains_key(from_node_key) {
                unresolved.push(from_node_key);
            }
            if !node_key_to_uid.contains_key(to_node_key) {
                unresolved.push(to_node_key);
            }
        }
        unresolved.sort_unstable();
        unresolved.dedup();
        if !unresolved.is_empty() {
            let m = query_for_node_keys(dgraph_client.clone(), &unresolved[..]).await;
            for (node_key, uid) in m.into_iter() {
                node_key_to_uid.insert(node_key, uid);
            }
        }

        let mut mutations = Vec::with_capacity(all_edges.len());
        for items in all_edges.into_iter().chunks_owned(DGRAPH_UPSERT_CHUNK_SIZE) {
            let mut mutation_units = vec![];

            for (from_node_key, to_node_key, edge_name) in items.iter() {
                let from_uid = node_key_to_uid.get(from_node_key);
                let to_uid = node_key_to_uid.get(to_node_key);
                let (from_uid, to_uid) = match (from_uid, to_uid) {
                    (Some(from_uid), Some(to_uid)) => (*from_uid, *to_uid),
                    (from_uid, to_uid) => {
                        tracing::error!(
                            message="Could not retrieve uids",
                            from_uid=?from_uid,
                            to_uid=?to_uid,
                            from_node_key=?from_node_key,
                            to_node_key=?to_node_key,
                        );
                        continue;
                    }
                };

                let (from_uid, to_uid) = (from_uid.to_string(), to_uid.to_string());
                let mutation_unit = MutationUnit::new(MutationUID::uid(&from_uid)).predicate(
                    &edge_name,
                    MutationPredicateValue::Edges(vec![MutationUID::uid(&to_uid)]),
                );
                mutation_units.push(mutation_unit);
            }
            let mutation = MutationBuilder::default()
                .set(mutation_units)
                .build()
                .unwrap();
            mutations.push(mutation);
        }

        futures::stream::iter(mutations.into_iter())
            .map(|mutation| {
                let dgraph_client = dgraph_client.clone();
                Self::enforce_transaction(move || {
                    let mut dgraph_mutation = dgraph_tonic::Mutation::new();
                    dgraph_mutation.set_set_json(&mutation.set).unwrap_or_else(
                        |e| tracing::error!(message="Failed to set json for mutation", error=?e),
                    );

                    let mut txn = dgraph_client.new_mutated_txn();
                    txn.mutate_and_commit_now(dgraph_mutation.clone())
                })
            })
            .buffer_unordered(DGRAPH_CONCURRENCY_UPSERTS)
            .collect::<Vec<_>>()
            .await;
    }

    async fn enforce_transaction<Factory, Txn>(f: Factory) -> dgraph_tonic::Response
    where
        Factory: FnMut() -> Txn + 'static + Unpin,
        Txn: std::future::Future<Output = Result<dgraph_tonic::Response, anyhow::Error>>,
    {
        let handle_upsert_err = UpsertErrorHandler {};
        let (response, attempts) = FutureRetry::new(f, handle_upsert_err)
            .await
            .expect("Surfaced an error despite retry strategy while performing an upsert.");

        tracing::info!(message = "Performed upsert", attempts = attempts);

        response
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty() && self.edges.is_empty()
    }
}

pub struct UpsertErrorHandler {}

impl futures_retry::ErrorHandler<anyhow::Error> for UpsertErrorHandler {
    type OutError = anyhow::Error;

    fn handle(&mut self, attempt: usize, e: anyhow::Error) -> RetryPolicy<Self::OutError> {
        let attempt = attempt as u64;
        tracing::warn!(
            message="Failed to enforce transaction",
            error=?e,
            attempt=?attempt,
        );
        match attempt {
            0..=2 => RetryPolicy::Repeat,
            t @ 2..=20 => RetryPolicy::WaitRetry(std::time::Duration::from_millis(10 * t as u64)),
            21..=u64::MAX => RetryPolicy::ForwardError(e),
        }
    }
}

async fn query_for_node_keys(
    dgraph_client: Arc<DgraphClient>,
    node_keys: &[&String],
) -> HashMap<String, u64> {
    let mut resolved_nodes = HashMap::new();

    let mut query_blocks = Vec::with_capacity(node_keys.len());
    for node_key in node_keys.iter() {
        let query_block = gen_node_key_query(node_key);
        query_blocks.push(query_block);
    }

    let query = QueryBuilder::default()
        .query_blocks(query_blocks)
        .build()
        .unwrap();

    let mut txn = dgraph_client.new_read_only_txn();
    let query_responses = txn
        .query(query.to_query_string())
        .await
        .expect("query failed");

    let query_responses: HashMap<String, Vec<HashMap<String, String>>> =
        serde_json::from_slice(&query_responses.json).expect("response failed to parse");

    for (_, query_response) in query_responses.into_iter() {
        let query_response = match query_response.as_slice() {
            [query_response] => query_response,
            [] => {
                tracing::error!(message = "Empty response for node_key");
                continue;
            }
            res => {
                tracing::error!(message = "Too many responses for node_key", count=?res.len());
                continue;
            }
        };
        let node_key = query_response.get("node_key");
        let uid = query_response.get("uid");
        let (node_key, uid) = match (node_key, uid) {
            (Some(node_key), Some(uid)) => (node_key, uid),
            (missing_key, missing_uid) => {
                tracing::error!(message="Unable to retrieve node_key and uid", node_key=?missing_key, uid=?missing_uid);
                continue;
            }
        };
        let uid = u64::from_str_radix(&uid[2..], 16).expect("uid is not valid hex");
        resolved_nodes.insert(node_key.to_owned(), uid);
    }

    resolved_nodes
}

fn gen_node_key_query(node_key: &str) -> QueryBlock {
    QueryBlockBuilder::default()
        .query_type(QueryBlockType::query())
        .root_filter(Condition::EQ(
            format!("node_key"),
            ConditionValue::string(node_key),
        ))
        .predicates(vec![
            Predicate::Field(Field::new("uid")),
            Predicate::Field(Field::new("node_key")),
        ])
        .first(1)
        .build()
        .unwrap()
}

fn uid_from_uids(
    node_key: &str,
    key_to_query_name: &HashMap<String, String>,
    uids: &HashMap<String, String>,
) -> Option<u64> {
    let query_name = key_to_query_name.get(node_key)?;
    let uid = uids.get(query_name)?;
    Some(u64::from_str_radix(&uid[2..], 16).expect("uid is not valid hex"))
}

fn extract_node_key_map_uid<'a>(
    dgraph_response: &'a serde_json::Value,
    node_key_map_to_uid: &mut HashMap<String, u64>,
) {
    let query_responses = dgraph_response.as_object().expect("Invalid response");

    for query_response in query_responses.values() {
        let query_response = query_response.as_array().expect("Invalid response");
        for query_response in query_response {
            let uid = query_response
                .get("uid")
                .expect("uid")
                .as_str()
                .expect("uid");
            let node_key = query_response
                .get("node_key")
                .expect("node_key")
                .as_str()
                .expect("node_key");

            // dgraph uids are hex encoded as '0x1b'
            let uid = u64::from_str_radix(&uid[2..], 16).expect("uid is not valid hex");
            node_key_map_to_uid.insert(node_key.to_owned(), uid);
        }
    }
}

impl MergedGraph {
    pub fn new() -> Self {
        Self {
            nodes: Default::default(),
            edges: Default::default(),
        }
    }

    pub fn add_node(&mut self, node: MergedNode) {
        match self.nodes.get_mut(&node.node_key) {
            Some(n) => n.merge(&node),
            None => {
                self.nodes.insert(node.clone_node_key(), node);
            }
        };
    }

    pub fn add_merged_edge(&mut self, edge: MergedEdge) {
        let from_node_key = edge.from_node_key.clone();
        let edge_list: &mut Vec<MergedEdge> = &mut self
            .edges
            .entry(from_node_key)
            .or_insert_with(|| MergedEdgeList {
                edges: Vec::with_capacity(1),
            })
            .edges;
        edge_list.push(edge);
    }

    pub fn add_edge(
        &mut self,
        edge_name: impl Into<String>,
        from_node_key: impl Into<String>,
        from_uid: impl Into<String>,
        to_node_key: impl Into<String>,
        to_uid: impl Into<String>,
    ) {
        let edge_name = edge_name.into();
        let from_node_key = from_node_key.into();
        let from_uid = from_uid.into();
        let to_node_key = to_node_key.into();
        let to_uid = to_uid.into();
        assert_ne!(from_node_key, to_node_key);
        assert_ne!(from_uid, to_uid);
        let edge = MergedEdge {
            from_node_key: from_node_key.clone(),
            from_uid: from_uid.clone(),
            to_node_key,
            to_uid,
            edge_name,
        };

        let edge_list: &mut Vec<MergedEdge> = &mut self
            .edges
            .entry(from_node_key)
            .or_insert_with(|| MergedEdgeList {
                edges: Vec::with_capacity(1),
            })
            .edges;
        edge_list.push(edge);
    }

    pub fn merge(&mut self, other: &Self) {
        for (node_key, other_node) in other.nodes.iter() {
            match self.nodes.get_mut(node_key) {
                Some(n) => n.merge(other_node),
                None => {
                    self.nodes.insert(node_key.clone(), other_node.clone());
                }
            };
        }

        for edge_list in other.edges.values() {
            for edge in edge_list.edges.iter() {
                self.add_edge(
                    edge.edge_name.clone(),
                    edge.from_node_key.clone(),
                    edge.from_uid.clone(),
                    edge.to_node_key.clone(),
                    edge.to_uid.clone(),
                );
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty() && self.edges.is_empty()
    }
}

impl NodeDescription {
    pub fn merge(&mut self, other: &Self) {
        debug_assert_eq!(self.node_type, other.node_type);
        debug_assert_eq!(self.node_key, other.node_key);
        for (prop_name, prop_value) in other.properties.iter() {
            match self.properties.get_mut(prop_name) {
                Some(self_prop) => self_prop.merge(prop_value),
                None => {
                    self.properties
                        .insert(prop_name.clone(), prop_value.clone());
                }
            }
        }
    }
    pub fn get_node_key(&self) -> &str {
        self.node_key.as_str()
    }

    pub fn clone_node_key(&self) -> String {
        self.node_key.clone()
    }
}

impl IdentifiedNode {
    pub fn into_json(self) -> serde_json::Value {
        let mut json_value = serde_json::Value::default();
        for (prop_name, prop_value) in self.properties {
            if let Some(prop_value) = prop_value.property {
                json_value[prop_name] = prop_value.into_json();
            }
        }

        json_value["node_key"] = self.node_key.into();
        json_value["dgraph.type"] = self.node_type.into();

        json_value
    }

    pub fn merge(&mut self, other: &Self) {
        debug_assert_eq!(self.node_type, other.node_type);
        debug_assert_eq!(self.node_key, other.node_key);
        for (prop_name, prop_value) in other.properties.iter() {
            match self.properties.get_mut(prop_name) {
                Some(self_prop) => self_prop.merge(prop_value),
                None => {
                    self.properties
                        .insert(prop_name.clone(), prop_value.clone());
                }
            }
        }
    }

    pub fn get_cache_identities_for_predicates(&self) -> Vec<Vec<u8>> {
        let mut predicate_cache_identities = Vec::with_capacity(self.properties.len());

        for (key, prop) in &self.properties {
            let prop_value = prop
                .property
                .as_ref()
                .map(Property::to_string)
                .unwrap_or_else(|| panic!("Invalid property on DynamicNode: {}", self.node_key));

            predicate_cache_identities.push(format!("{}:{}:{}", &self.node_key, key, prop_value));
        }

        predicate_cache_identities
            .into_iter()
            .map(|item| item.into_bytes())
            .collect()
    }
}

impl NodeProperty {
    pub fn merge(&mut self, other: &Self) {
        match (self.property.as_mut(), other.property.as_ref()) {
            (
                Some(ProtoIncrementOnlyUintProp(ref mut self_prop)),
                Some(ProtoIncrementOnlyUintProp(ref other_prop)),
            ) => {
                *self_prop = std::cmp::max(*self_prop, *other_prop);
            }
            (
                Some(ProtoImmutableUintProp(ref mut self_prop)),
                Some(ProtoImmutableUintProp(ref other_prop)),
            ) => {
                debug_assert_eq!(*self_prop, *other_prop);
            }
            (
                Some(ProtoDecrementOnlyUintProp(ref mut self_prop)),
                Some(ProtoDecrementOnlyUintProp(ref other_prop)),
            ) => {
                *self_prop = std::cmp::min(*self_prop, *other_prop);
            }
            (
                Some(ProtoDecrementOnlyIntProp(ref mut self_prop)),
                Some(ProtoDecrementOnlyIntProp(ref other_prop)),
            ) => {
                *self_prop = std::cmp::min(*self_prop, *other_prop);
            }
            (
                Some(ProtoIncrementOnlyIntProp(ref mut self_prop)),
                Some(ProtoIncrementOnlyIntProp(ref other_prop)),
            ) => {
                *self_prop = std::cmp::max(*self_prop, *other_prop);
            }
            (
                Some(ProtoImmutableIntProp(ref mut self_prop)),
                Some(ProtoImmutableIntProp(ref other_prop)),
            ) => {
                debug_assert_eq!(*self_prop, *other_prop);
            }
            (
                Some(ProtoImmutableStrProp(ref mut self_prop)),
                Some(ProtoImmutableStrProp(ref other_prop)),
            ) => {
                debug_assert_eq!(*self_prop, *other_prop);
            }
            (None, op) => {
                debug_assert!(false, "Unhandled property merge, self is None: {:?}", op);
                tracing::warn!("Unhandled property merge, self is None: {:?}", op);
            }
            (p, None) => {
                debug_assert!(false, "Unhandled property merge, other is None: {:?}", p);
                tracing::warn!("Unhandled property merge, other is None: {:?}", p);
            }
            // technically we could improve type safety here by exhausting the combinations,
            // but I'm not going to type that all out right now
            (p, op) => {
                debug_assert!(false, "Unhandled property merge: {:?} {:?}", p, op);
                tracing::warn!("Unhandled property merge: {:?} {:?}", p, op);
            }
        }
    }
}

impl From<Static> for IdStrategy {
    fn from(strategy: Static) -> IdStrategy {
        IdStrategy {
            strategy: Some(id_strategy::Strategy::Static(strategy)),
        }
    }
}

impl From<Session> for IdStrategy {
    fn from(strategy: Session) -> IdStrategy {
        IdStrategy {
            strategy: Some(id_strategy::Strategy::Session(strategy)),
        }
    }
}
impl std::string::ToString for Property {
    fn to_string(&self) -> String {
        match self {
            ProtoIncrementOnlyUintProp(increment_only_uint_prop) => {
                increment_only_uint_prop.to_string()
            }
            ProtoImmutableUintProp(immutable_uint_prop) => immutable_uint_prop.to_string(),
            ProtoDecrementOnlyUintProp(decrement_only_uint_prop) => {
                decrement_only_uint_prop.to_string()
            }
            ProtoDecrementOnlyIntProp(decrement_only_int_prop) => {
                decrement_only_int_prop.to_string()
            }
            ProtoIncrementOnlyIntProp(increment_only_int_prop) => {
                increment_only_int_prop.to_string()
            }
            ProtoImmutableIntProp(immutable_int_prop) => immutable_int_prop.to_string(),
            ProtoImmutableStrProp(immutable_str_prop) => immutable_str_prop.to_string(),
        }
    }
}
impl std::string::ToString for NodeProperty {
    fn to_string(&self) -> String {
        let prop = match &self.property {
            Some(node_property::Property::IncrementOnlyUintProp(increment_only_uint_prop)) => {
                increment_only_uint_prop.to_string()
            }
            Some(node_property::Property::ImmutableUintProp(immutable_uint_prop)) => {
                immutable_uint_prop.to_string()
            }
            Some(node_property::Property::DecrementOnlyUintProp(decrement_only_uint_prop)) => {
                decrement_only_uint_prop.to_string()
            }
            Some(node_property::Property::DecrementOnlyIntProp(decrement_only_int_prop)) => {
                decrement_only_int_prop.to_string()
            }
            Some(node_property::Property::IncrementOnlyIntProp(increment_only_int_prop)) => {
                increment_only_int_prop.to_string()
            }
            Some(node_property::Property::ImmutableIntProp(immutable_int_prop)) => {
                immutable_int_prop.to_string()
            }
            Some(node_property::Property::ImmutableStrProp(immutable_str_prop)) => {
                immutable_str_prop.to_string()
            }
            None => panic!("Invalid property : {:?}", self),
        };
        prop
    }
}

impl Property {
    pub fn into_json(self) -> serde_json::Value {
        match self {
            ProtoIncrementOnlyUintProp(increment_only_uint_prop) => increment_only_uint_prop.into(),
            ProtoImmutableUintProp(immutable_uint_prop) => immutable_uint_prop.into(),
            ProtoDecrementOnlyUintProp(decrement_only_uint_prop) => decrement_only_uint_prop.into(),
            ProtoDecrementOnlyIntProp(decrement_only_int_prop) => decrement_only_int_prop.into(),
            ProtoIncrementOnlyIntProp(increment_only_int_prop) => increment_only_int_prop.into(),
            ProtoImmutableIntProp(immutable_int_prop) => immutable_int_prop.into(),
            ProtoImmutableStrProp(immutable_str_prop) => immutable_str_prop.into(),
        }
    }
}

impl NodeDescription {
    pub fn get_property(&self, name: impl AsRef<str>) -> Option<&NodeProperty> {
        self.properties.get(name.as_ref())
    }

    pub fn set_property(&mut self, name: impl Into<String>, value: impl Into<NodeProperty>) {
        self.properties.insert(name.into(), value.into().into());
    }

    pub fn set_key(&mut self, key: String) {
        self.node_key = key;
    }
}

impl<T> From<T> for NodeProperty
where
    T: Into<Property>,
{
    fn from(t: T) -> Self {
        NodeProperty {
            property: Some(t.into()),
        }
    }
}

impl From<NodeDescription> for IdentifiedNode {
    fn from(n: NodeDescription) -> Self {
        IdentifiedNode {
            properties: n.properties,
            node_key: n.node_key,
            node_type: n.node_type,
        }
    }
}

impl MergedNode {
    pub fn from(n: IdentifiedNode, uid: u64) -> Self {
        Self {
            uid,
            properties: n.properties,
            node_key: n.node_key,
            node_type: n.node_type,
        }
    }

    pub fn merge(&mut self, other: &Self) {
        debug_assert_eq!(self.node_type, other.node_type);
        debug_assert_eq!(self.node_key, other.node_key);
        for (prop_name, prop_value) in other.properties.iter() {
            match self.properties.get_mut(prop_name) {
                Some(self_prop) => self_prop.merge(prop_value),
                None => {
                    self.properties
                        .insert(prop_name.clone(), prop_value.clone());
                }
            }
        }
    }

    pub fn get_node_key(&self) -> &str {
        self.node_key.as_str()
    }

    pub fn clone_node_key(&self) -> String {
        self.node_key.clone()
    }
}

impl IdentifiedNode {
    pub fn into(self, uid: u64) -> MergedNode {
        MergedNode {
            uid,
            properties: self.properties,
            node_key: self.node_key,
            node_type: self.node_type,
        }
    }
    pub fn get_node_key(&self) -> &str {
        self.node_key.as_str()
    }

    pub fn clone_node_key(&self) -> String {
        self.node_key.clone()
    }
}

// We use separate types here because it makes a lot of the codegen easier
pub struct IncrementOnlyIntProp(pub i64);

pub struct DecrementOnlyIntProp(pub i64);

pub struct ImmutableIntProp(pub i64);

pub struct IncrementOnlyUintProp(pub u64);

pub struct DecrementOnlyUintProp(pub u64);

pub struct ImmutableUintProp(pub u64);

pub struct ImmutableStrProp(pub String);

impl_from_for!(
    ImmutableUintProp,
    0,
    u64,
    u32,
    u16,
    u8,
    &u64,
    &u32,
    &u16,
    &u8
);
impl_from_for!(
    IncrementOnlyUintProp,
    0,
    u64,
    u32,
    u16,
    u8,
    &u64,
    &u32,
    &u16,
    &u8
);
impl_from_for!(
    DecrementOnlyUintProp,
    0,
    u64,
    u32,
    u16,
    u8,
    &u64,
    &u32,
    &u16,
    &u8
);
impl_from_for!(
    ImmutableIntProp,
    0,
    i64,
    i32,
    i16,
    i8,
    &i64,
    &i32,
    &i16,
    &i8
);
impl_from_for!(
    IncrementOnlyIntProp,
    0,
    i64,
    i32,
    i16,
    i8,
    &i64,
    &i32,
    &i16,
    &i8
);
impl_from_for!(
    DecrementOnlyIntProp,
    0,
    i64,
    i32,
    i16,
    i8,
    &i64,
    &i32,
    &i16,
    &i8
);
impl_from_for!(ImmutableStrProp, 0, String, &String, &str);
impl_from_for!(ImmutableStrProp, Property);
impl_from_for!(IncrementOnlyIntProp, Property);
impl_from_for!(DecrementOnlyIntProp, Property);
impl_from_for!(ImmutableIntProp, Property);
impl_from_for!(IncrementOnlyUintProp, Property);
impl_from_for!(DecrementOnlyUintProp, Property);
impl_from_for!(ImmutableUintProp, Property);

impl_as! {NodeProperty, as_increment_only_uint, u64, node_property::Property::IncrementOnlyUintProp}
impl_as! {NodeProperty, as_immutable_uint, u64, node_property::Property::ImmutableUintProp}
impl_as! {NodeProperty, as_decrement_only_uint, u64, node_property::Property::DecrementOnlyUintProp}
impl_as! {NodeProperty, as_decrement_only_int, i64, node_property::Property::DecrementOnlyIntProp}
impl_as! {NodeProperty, as_increment_only_int, i64, node_property::Property::IncrementOnlyIntProp}
impl_as! {NodeProperty, as_immutable_int, i64, node_property::Property::ImmutableIntProp}
impl_as! {NodeProperty, as_immutable_str, &str, node_property::Property::ImmutableStrProp }

#[cfg(feature = "integration")]
pub mod test {
    use super::*;
    use dgraph_query_lib::schema::{
        Indexing, PredicateDefinition, PredicateType, Schema, SchemaDefinition,
    };
    use dgraph_query_lib::EdgeBuilder;
    use dgraph_query_lib::ToQueryString;
    use dgraph_tonic::Query;
    use std::sync::Once;

    async fn query_for_uid(dgraph_client: Arc<DgraphClient>, node_key: &str) -> u64 {
        let query_block = QueryBlockBuilder::default()
            .query_type(QueryBlockType::query())
            .root_filter(Condition::EQ(
                format!("node_key"),
                ConditionValue::string(node_key),
            ))
            .predicates(vec![Predicate::Field(Field::new("uid"))])
            .first(1)
            .build()
            .unwrap();

        let query = QueryBuilder::default()
            .query_blocks(vec![query_block])
            .build()
            .unwrap();

        let mut txn = dgraph_client.new_read_only_txn();
        let response = txn
            .query(query.to_query_string())
            .await
            .expect("query failed");

        let m: HashMap<String, Vec<HashMap<String, String>>> =
            serde_json::from_slice(&response.json).expect("response failed to parse");
        let m = m.into_iter().next().unwrap().1;
        debug_assert!((m.len() == 1) || (m.len() == 0));

        let uid = &m[0]["uid"][2..];
        let uid = u64::from_str_radix(uid, 16).expect("uid is not valid hex");
        uid
    }

    async fn query_for_edge(
        dgraph_client: Arc<DgraphClient>,
        from_uid: u64,
        to_uid: u64,
        edge_name: &str,
    ) -> serde_json::Value {
        let edge = Predicate::Edge(
            EdgeBuilder::default()
                .name(edge_name.to_string())
                .predicates(vec![
                    Predicate::Field(Field::new("uid")),
                    Predicate::Field(Field::new("dgraph.type").alias("dgraph_type")),
                ])
                .build()
                .unwrap(),
        );

        let query_block = QueryBlockBuilder::default()
            .query_type(QueryBlockType::query())
            .root_filter(Condition::uid(&from_uid.to_string()))
            .predicates(vec![Predicate::Field(Field::new("uid")), edge])
            .first(1)
            .build()
            .unwrap();

        let query = QueryBuilder::default()
            .query_blocks(vec![query_block])
            .build()
            .unwrap();

        let mut txn = dgraph_client.new_read_only_txn();
        let response = txn
            .query(query.to_query_string())
            .await
            .expect("query failed");

        serde_json::from_slice(&response.json).expect("response failed to parse")
    }

    fn init_test_env() {
        static START: Once = Once::new();
        START.call_once(|| {
            let filter = tracing_subscriber::EnvFilter::from_default_env();
            tracing_subscriber::fmt()
                .with_env_filter(filter)
                .with_writer(std::io::stdout)
                .init();
            let schema = Schema::new()
                .add_definition(
                    SchemaDefinition::new("ExampleNode")
                        .add_predicate(
                            PredicateDefinition::new("example_id", PredicateType::INT)
                                .add_index(Indexing::INT),
                        )
                        .add_predicate(
                            PredicateDefinition::new("node_key", PredicateType::String)
                                .add_index(Indexing::EXACT)
                                .upsert(),
                        )
                        .add_predicate(
                            PredicateDefinition::new("example_name", PredicateType::String)
                                .add_index(Indexing::TRIGRAM),
                        )
                        .add_predicate(
                            PredicateDefinition::new("to_many_edge", PredicateType::UIDArray)
                        )
                        .add_predicate(
                            PredicateDefinition::new("to_single_edge", PredicateType::UID)
                        ),
                )
                .to_string();

            std::thread::spawn(move || {
                let mut rt  = tokio::runtime::Runtime::new()
                    .expect("failed to init runtime");
                rt.block_on(async {
                    let dgraph_client = DgraphClient::new("http://127.0.0.1:9080")
                        .expect("Failed to create dgraph client.");

                    dgraph_client.alter(dgraph_tonic::Operation {
                        drop_all: true,
                        ..Default::default()
                    }).await.expect("alter failed");

                    dgraph_client.alter(dgraph_tonic::Operation {
                        schema,
                        ..Default::default()
                    }).await.expect("alter failed");
                });
            }).join().expect("provision failed");
        });
    }

    #[tokio::test(threaded_scheduler)]
    async fn test_upsert_edge_and_retrieve() -> Result<(), Box<dyn std::error::Error>> {
        init_test_env();
        let mut identified_graph = IdentifiedGraph::new();
        let mut merged_graph = MergedGraph::new();
        let dgraph_client =
            DgraphClient::new("http://127.0.0.1:9080").expect("Failed to create dgraph client.");
        let dgraph_client = std::sync::Arc::new(dgraph_client);
        let mut properties = HashMap::new();
        properties.insert(
            "example_name".to_string(),
            ProtoImmutableStrProp("foobar".to_string()).into(),
        );
        let n0 = IdentifiedNode {
            node_key: "example-node-key".to_string(),
            node_type: "ExampleNode".to_string(),
            properties,
        };

        let mut properties = HashMap::new();
        properties.insert(
            "example_name".to_string(),
            ProtoImmutableStrProp("baz".to_string()).into(),
        );

        let n1 = IdentifiedNode {
            node_key: "someother-node-key".to_string(),
            node_type: "Process".to_string(),
            properties,
        };

        identified_graph.add_node(n0);
        identified_graph.add_node(n1);

        identified_graph.add_edge(
            "to_many_edge".to_string(),
            "example-node-key".to_string(),
            "someother-node-key".to_string(),
        );

        identified_graph.add_edge(
            "to_single_edge".to_string(),
            "someother-node-key".to_string(),
            "example-node-key".to_string(),
        );


        identified_graph
            .perform_upsert_into(dgraph_client.clone(), &mut merged_graph)
            .await;

        let node_uid_0 = query_for_uid(dgraph_client.clone(), "example-node-key").await;
        let node_uid_1 = query_for_uid(dgraph_client.clone(), "someother-node-key").await;
        assert_ne!(node_uid_0, node_uid_1);
        assert_ne!(node_uid_0, 0);
        assert_ne!(node_uid_1, 0);

        let to_many_res =
            query_for_edge(dgraph_client.clone(), node_uid_0, node_uid_1, "to_many_edge").await;

        let to_single_res =
            query_for_edge(dgraph_client.clone(), node_uid_1, node_uid_0, "to_single_edge").await;

        let to_many_res = to_many_res
            .as_object()
            .expect("to_many_res.as_object")
            .values()
            .next()
            .expect("to_many_res empty array");
        let to_single_res = to_single_res
            .as_object()
            .expect("to_single_res.as_object")
            .values()
            .next()
            .expect("to_single_res empty array");

        let tm_from = to_many_res[0]["uid"].as_str().expect("tm_from");
        let tm_to = to_many_res[0]["to_many_edge"][0]["uid"].as_str().expect("tm_to");

        let ts_from = to_single_res[0]["uid"].as_str().expect("ts_from");
        let ts_to = to_single_res[0]["to_single_edge"]["uid"].as_str().expect("ts_to");

        assert_eq!(tm_from, ts_to);
        assert_eq!(tm_to, ts_from);
        Ok(())
    }


    #[tokio::test(threaded_scheduler)]
    async fn test_upsert_idempotency() -> Result<(), Box<dyn std::error::Error>> {
        init_test_env();

        let dgraph_client =
            DgraphClient::new("http://127.0.0.1:9080").expect("Failed to create dgraph client.");
        let dgraph_client = std::sync::Arc::new(dgraph_client);

        let node_key = "test_upsert_idempotency-example-node-key";
        let mut properties = HashMap::new();
        properties.insert(
            "example_name".to_string(),
            ProtoImmutableStrProp("foobar".to_string()).into(),
        );
        let n0 = IdentifiedNode {
            node_key: node_key.to_string(),
            node_type: "ExampleNode".to_string(),
            properties,
        };

        let upsert_futs: Vec<_> = (0..10).map(|_| {
            let dgraph_client = dgraph_client.clone();
            let n0 = n0.clone();
            async move {
                let mut identified_graph = IdentifiedGraph::new();
                identified_graph.add_node(n0);
                let mut merged_graph = MergedGraph::new();

                identified_graph
                    .perform_upsert_into(dgraph_client.clone(), &mut merged_graph)
                    .await;
                merged_graph
            }
        }).collect();

        let mut merged_graphs = Vec::with_capacity(upsert_futs.len());
        for upsert_fut in upsert_futs.into_iter() {
            merged_graphs.push(upsert_fut.await);
        }
        for merged_graph in merged_graphs {
            assert_eq!(merged_graph.nodes.len(), 1);
        }

        // If we query for multiple nodes by node_key we should only ever receive one
        let query_block = QueryBlockBuilder::default()
            .query_type(QueryBlockType::query())
            .root_filter(Condition::EQ(
                format!("node_key"),
                ConditionValue::string(node_key),
            ))
            .predicates(vec![Predicate::Field(Field::new("uid"))])
            .first(2)
            .build()
            .unwrap();

        let query = QueryBuilder::default()
            .query_blocks(vec![query_block])
            .build()
            .unwrap();

        let mut txn = dgraph_client.new_read_only_txn();
        let response = txn
            .query(query.to_query_string())
            .await
            .expect("query failed");

        let m: HashMap<String, Vec<HashMap<String, String>>> =
            serde_json::from_slice(&response.json).expect("response failed to parse");
        let m = m.into_iter().next().unwrap().1;
        debug_assert_eq!(m.len(), 1);
        Ok(())
    }
}
