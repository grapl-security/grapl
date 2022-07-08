use std::{
    collections::HashSet,
    sync::{
        atomic::{
            AtomicBool,
            Ordering,
        },
        Arc,
        Mutex,
    },
};
use rust_proto::graplinc::grapl::api::graph_query_service::v1beta1::messages::QueryId;

use rust_proto::graplinc::grapl::common::v1beta1::types::{
    EdgeName,
};

// We should have the short circuit logic get shared between tasks
// so that parallel queries in WithUid queries can short circuit

#[derive(Clone)]
pub struct Visited {
    short_circuit: Arc<AtomicBool>,
    already_visited: Arc<Mutex<HashSet<(QueryId, EdgeName, QueryId)>>>,
}

impl Visited {
    pub fn new() -> Self {
        Self {
            short_circuit: Arc::new(AtomicBool::new(false)),
            already_visited: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    pub fn get_short_circuit(&self) -> bool {
        self.short_circuit.as_ref().load(Ordering::Acquire)
    }

    pub fn set_short_circuit(&self) {
        self.short_circuit.as_ref().store(true, Ordering::Release)
    }

    pub fn check_and_add(&self, src: QueryId, edge_name: EdgeName, dst: QueryId) -> bool {
        let already_visited =
            (*self.already_visited.lock().unwrap()).contains(&(src, edge_name.clone(), dst));
        self.add(src, edge_name, dst);
        already_visited
    }

    pub fn check(&self, src: QueryId, edge_name: EdgeName, dst: QueryId) -> bool {
        (*self.already_visited.lock().unwrap()).contains(&(src, edge_name.into(), dst))
    }

    pub fn add(&self, src: QueryId, edge_name: EdgeName, dst: QueryId) {
        (*self.already_visited.lock().unwrap()).insert((src, edge_name.into(), dst));
    }
}
