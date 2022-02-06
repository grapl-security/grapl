use std::{
    collections::{
        HashSet,
        VecDeque,
    },
    iter::Iterator,
};

use crate::NodeCell;

pub struct NodeIterator {
    to_visit: VecDeque<NodeCell>,
    already_visited: HashSet<u128>,
}

impl NodeIterator {
    pub fn new(node: NodeCell) -> Self {
        let mut to_visit = VecDeque::with_capacity(1);

        to_visit.push_back(node);
        Self {
            to_visit,
            already_visited: HashSet::new(),
        }
    }
}

impl Iterator for NodeIterator {
    type Item = NodeCell;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(next) = self.to_visit.pop_front() {
            let inner = next.borrow();
            if self.already_visited.contains(&inner.id) {
                continue;
            }
            self.already_visited.insert(inner.id);
            for n in inner.edge_filters.values().flatten() {
                if self.already_visited.contains(&n.get_id()) {
                    continue;
                }
                self.to_visit.push_back(n.clone());
            }
            drop(inner);
            return Some(next);
        }
        None
    }
}
