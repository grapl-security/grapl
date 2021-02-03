use log::warn;
use serde_json::{json,
                 Value};

use crate::graph_description::IpAddress;
use crate::node::NodeT;
use dgraph_query_lib::mutation::{MutationUnit, MutationPredicateValue};

impl IpAddress {
    pub fn new(
        ip_address: impl Into<String>,
        first_seen_timestamp: u64,
        last_seen_timestamp: u64,
    ) -> Self {
        let ip_address = ip_address.into();

        Self {
            node_key: ip_address.to_string(),
            ip_address,
            first_seen_timestamp,
            last_seen_timestamp,
        }
    }

    pub fn into_json(self) -> Value {
        let mut j = json!({
            "node_key": self.node_key,
            "dgraph.type": "IpAddress",
            "ip_address": self.ip_address,
        });

        if self.first_seen_timestamp != 0 {
            j["first_seen_timestamp"] = self.first_seen_timestamp.into();
        }

        if self.last_seen_timestamp != 0 {
            j["last_seen_timestamp"] = self.last_seen_timestamp.into();
        }

        j
    }
}

impl NodeT for IpAddress {
    fn get_asset_id(&self) -> Option<&str> {
        None
    }

    fn set_asset_id(&mut self, _asset_id: impl Into<String>) {
        panic!("Can not set asset_id on IpAddress");
    }

    fn get_node_key(&self) -> &str {
        &self.node_key
    }

    fn set_node_key(&mut self, node_key: impl Into<String>) {
        self.node_key = node_key.into();
    }

    fn merge(&mut self, other: &Self) -> bool {
        if self.node_key != other.node_key {
            warn!("Attempted to merge two IpAddress Nodes with differing node_keys");
            return false;
        }

        if self.ip_address != other.ip_address {
            warn!("Attempted to merge two IpAddress Nodes with differing IPs");
            return false;
        }

        let mut merged = false;

        if other.first_seen_timestamp != 0 && self.first_seen_timestamp > other.first_seen_timestamp
        {
            self.first_seen_timestamp = other.first_seen_timestamp;
            merged = true;
        }

        if other.last_seen_timestamp != 0 && self.last_seen_timestamp < other.last_seen_timestamp {
            self.last_seen_timestamp = other.last_seen_timestamp;
            merged = true;
        }

        merged
    }

    fn merge_into(&mut self, other: Self) -> bool {
        self.merge(&other)
    }

    fn attach_predicates_to_mutation_unit(&self, mutation_unit: &mut MutationUnit) {
        mutation_unit.predicate_ref("node_key", MutationPredicateValue::string(&self.node_key));
        mutation_unit.predicate_ref("ip_address", MutationPredicateValue::string(&self.ip_address));
        mutation_unit.predicate_ref("dgraph.type", MutationPredicateValue::string("IpAddress"));

        if self.first_seen_timestamp != 0 {
            mutation_unit.predicate_ref("first_seen_timestamp", MutationPredicateValue::Number(self.first_seen_timestamp as i64));
        }

        if self.last_seen_timestamp != 0 {
            mutation_unit.predicate_ref("last_seen_timestamp", MutationPredicateValue::Number(self.last_seen_timestamp as i64));
        }
    }
}
