use std::{
    borrow::Borrow,
    cell::{
        Ref,
        RefCell,
    },
    collections::{
        hash_map::Entry,
        HashMap,
        HashSet,
        VecDeque,
    },
    hash::Hash,
    marker::PhantomData,
    rc::{
        Rc,
        Weak,
    },
};

use rust_proto_new::graplinc::grapl::common::v1beta1::types::{
    EdgeName,
    Uid,
};

use crate::node_view::Node;

#[derive(Debug, Default)]
pub struct Graph {
    pub nodes: HashMap<Uid, Node>,
    pub edges: HashMap<(Uid, EdgeName), HashSet<Uid>>,
}

impl Graph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: HashMap::new(),
        }
    }

    pub fn new_node(&mut self, uid: Uid, query_id: u64) -> &mut Node {
        self.nodes
            .entry(uid)
            .or_insert_with(|| Node::new(uid, query_id))
    }

    pub fn add_node(&mut self, node: Node) {
        match self.nodes.entry(node.uid) {
            Entry::Occupied(n) => {
                n.into_mut().merge(node);
            }
            Entry::Vacant(n) => {
                n.insert(node);
            }
        }
    }

    pub fn find_node_by_query_id(&self, query_id: u64) -> Option<&Node> {
        for node in self.nodes.values() {
            if node.query_ids.contains(&query_id) {
                return Some(node);
            }
        }
        None
    }

    pub fn add_edge(&mut self, from: Uid, edge_name: EdgeName, to: Uid) {
        self.edges
            .entry((from, edge_name))
            .or_insert_with(|| HashSet::new())
            .insert(to);
    }

    pub fn add_edges(&mut self, src_uid: Uid, edge_name: EdgeName, dst_uids: HashSet<Uid>) {
        self.edges
            .entry((src_uid, edge_name))
            .or_insert_with(|| HashSet::new())
            .extend(dst_uids);
    }

    pub fn get_node(&self, uid: Uid) -> Option<&Node> {
        self.nodes.get(&uid)
    }

    pub fn get_edges(&self, from: Uid) -> impl Iterator<Item = (&EdgeName, &HashSet<Uid>)> {
        self.edges
            .iter()
            .filter(move |(key, _)| key.0 == from)
            .map(|(key, value)| (&key.1, value))
    }

    pub fn merge(&mut self, other: Self) {
        for (_, node) in other.nodes.into_iter() {
            self.add_node(node);
        }

        for ((src_uid, edge_name), dst_uids) in other.edges.into_iter() {
            self.add_edges(src_uid.clone(), edge_name.clone(), dst_uids.clone());
        }
    }

    pub fn get_nodes(&self) -> &HashMap<Uid, Node> {
        &self.nodes
    }
}
