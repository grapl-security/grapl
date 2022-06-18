use std::{
    cell::Ref,
    collections::{
        HashMap,
        HashSet,
        VecDeque,
    },
    fmt::{
        Debug,
        Formatter,
    },
    marker::PhantomData,
};

use rust_proto_new::graplinc::grapl::common::v1beta1::types::{
    PropertyName,
    Uid,
};

use crate::graph_view::Graph;

// pub struct InnerNode {
//     pub uid: i64,
//     pub query_ids: HashSet<u64>,
//     pub string_properties: HashMap<String, String>,
// }
//
// pub struct GraphView {
//     nodes: HashMap<i64, InnerNode>,
//     edges: HashMap<(i64, String), HashSet<i64>>,
// }

pub struct Node {
    pub uid: Uid,
    pub query_ids: HashSet<u64>,
    pub string_properties: HashMap<PropertyName, String>,
}

impl Debug for Node {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Node")
            .field("uid", &self.uid)
            .field("query_ids", &self.query_ids)
            .field("string_properties", &self.string_properties)
            .finish()
    }
}

impl Node {
    pub fn new(uid: Uid, query_id: u64) -> Self {
        Self {
            uid,
            query_ids: HashSet::from([query_id]),
            string_properties: Default::default(),
        }
    }

    pub fn merge(&mut self, other: Node) {
        assert_eq!(self.uid, other.uid);
        for (key, value) in other.string_properties {
            self.string_properties.insert(key, value);
        }

        self.query_ids.extend(other.query_ids);
    }

    pub fn add_string_property(&mut self, key: PropertyName, value: String) {
        self.string_properties.insert(key, value);
    }
}
