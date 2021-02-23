#![allow(warnings)]
pub use crate::{graph_description::*, node_property::Property};
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
use log::{error, info, warn};

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
        self.upsert_nodes(dgraph_client.clone(), merged_graph).await;
        self.upsert_edges(dgraph_client).await;
    }

    async fn upsert_nodes(&self, dgraph_client: Arc<DgraphClient>, merged_graph: &mut MergedGraph) {
        let unique_id = 0;
        let mut node_upserts = Vec::with_capacity(self.nodes.len());
        for node in self.nodes.values() {
            let (query, upserts) = upsert::build_upserts(
                unique_id,
                &node.node_key,
                &node.node_type,
                &node.properties,
            );
            node_upserts.push((query, upserts));
        }

        let responses: Vec<dgraph_tonic::Response> = futures::stream::iter(
            node_upserts
                .into_iter()
                .chunks_owned(DGRAPH_UPSERT_CHUNK_SIZE),
        )
        .map(|upsert_chunk| {
            let mut combined_query = String::new();
            let mut all_mutations = Vec::new();
            for (query_block, mutations) in upsert_chunk.iter() {
                combined_query.push_str(&query_block);
                all_mutations.extend_from_slice(mutations);
            }

            let combined_query = format!(r"
            query() {{
                {}
            }}
            ", combined_query);

            Self::enforce_transaction(dgraph_client.clone(), combined_query, all_mutations)
        })
        .buffer_unordered(DGRAPH_CONCURRENCY_UPSERTS)
        .collect::<Vec<_>>()
        .await;

        for response in responses {
            let query_responses: serde_json::Value = match serde_json::from_slice(&response.json) {
                Ok(response) => response,
                Err(e) => {
                    tracing::error!(message="Failed to parse JSON response for upsert", error=?e);
                    continue
                }
            };

            let query_responses = query_responses.as_object()
                .expect("Invalid response");

            for query_response in query_responses.values() {
                let query_response = query_response.as_array()
                    .expect("Invalid response");
                for query_response in query_response {
                    let uid = query_response.get("uid")
                        .expect("uid")
                        .as_str()
                        .expect("uid");
                    let node_key = query_response.get("node_key")
                        .expect("node_key")
                        .as_str()
                        .expect("node_key");
                    println!("{}  {}", uid, node_key);
                }
            }

        }
    }

    async fn upsert_edges(&self, dgraph_client: Arc<DgraphClient>) {
        // let all_edges: Vec<_> = self
        //     .edges
        //     .iter()
        //     .flat_map(|(_, EdgeList { edges })| edges)
        //     .map(
        //         |Edge {
        //              from_node_key,
        //              to_node_key,
        //              edge_name,
        //          }| {
        //             (
        //                 from_node_key.clone(),
        //                 to_node_key.clone(),
        //                 edge_name.clone(),
        //             )
        //         },
        //     )
        //     .collect();
        //
        // futures::stream::iter(all_edges.into_iter().chunks_owned(DGRAPH_UPSERT_CHUNK_SIZE))
        //     .map(|items| {
        //
        //         Self::enforce_transaction(dgraph_client.clone(), upsert)
        //     })
        //     .buffer_unordered(DGRAPH_CONCURRENCY_UPSERTS)
        //     .collect::<Vec<_>>()
        //     .await;
    }

    async fn enforce_transaction(client: Arc<DgraphClient>, q: String, mutations: Vec<dgraph_tonic::Mutation>) -> dgraph_tonic::Response {
        let (response, attempts) = FutureRetry::new(
            || {
                let client = client.clone();
                let mut txn = client.new_mutated_txn();
                txn.upsert_and_commit_now(q.clone(), mutations.clone())
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

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty() && self.edges.is_empty()
    }
}

impl MergedGraph {
    pub fn new() -> Self {
        Self {
            nodes: Default::default(),
            edges: Default::default(),
        }
    }

    pub fn add_node_without_edges<N>(&mut self, node: N)
    where
        N: Into<MergedNode>,
    {
        let node = node.into();
        let key = node.clone_node_key();

        self.nodes.insert(key.to_string(), node);
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

    #[tokio::test]
    async fn test_upsert() -> Result<(), Box<dyn std::error::Error>> {
        let filter = tracing_subscriber::EnvFilter::from_default_env();
        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .with_writer(std::io::stdout)
            .init();

        let mut identified_graph = IdentifiedGraph::new();
        let mut merged_graph = MergedGraph::new();
        let dgraph_client = DgraphClient::new(vec!["localhost:9080"]).expect("Failed to create dgraph client.");
        let dgraph_client = std::sync::Arc::new(dgraph_client);
        let mut properties = HashMap::new();

        let n = IdentifiedNode {
            node_key: "example-node-key".to_string(),
            node_type: "Process".to_string(),
            properties,
        };

        identified_graph.add_node(n);

        let d = identified_graph.upsert_nodes(dgraph_client, &mut merged_graph).await;

        dbg!(&d);

        Ok(())
    }
}
