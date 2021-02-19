use std::convert::TryFrom;

use log::warn;
use serde_json::{json,
                 Value};
use uuid::Uuid;

use crate::error::Error;
use crate::graph_description::NetworkConnection;
use crate::node::NodeT;
use dgraph_query_lib::mutation::{MutationUnit, MutationPredicateValue};

pub enum NetworkConnectionState {
    Created,
    Existing,
    Terminated,
}

impl From<NetworkConnectionState> for u32 {
    fn from(p: NetworkConnectionState) -> u32 {
        match p {
            NetworkConnectionState::Created => 1,
            NetworkConnectionState::Terminated => 2,
            NetworkConnectionState::Existing => 3,
        }
    }
}

impl TryFrom<u32> for NetworkConnectionState {
    type Error = Error;

    fn try_from(p: u32) -> Result<NetworkConnectionState, Error> {
        match p {
            1 => Ok(NetworkConnectionState::Created),
            2 => Ok(NetworkConnectionState::Terminated),
            3 => Ok(NetworkConnectionState::Existing),
            _ => Err(Error::InvalidNetworkConnectionState(p)),
        }
    }
}

impl NetworkConnection {
    pub fn new(
        src_ip_address: impl Into<String>,
        dst_ip_address: impl Into<String>,
        protocol: impl Into<String>,
        src_port: u16,
        dst_port: u16,
        state: NetworkConnectionState,
        created_timestamp: u64,
        terminated_timestamp: u64,
        last_seen_timestamp: u64,
    ) -> Self {
        let src_ip_address = src_ip_address.into();
        let dst_ip_address = dst_ip_address.into();
        let protocol = protocol.into();

        Self {
            node_key: Uuid::new_v4().to_string(),
            src_ip_address,
            dst_ip_address,
            protocol,
            src_port: src_port as u32,
            dst_port: dst_port as u32,
            state: state.into(),
            created_timestamp,
            terminated_timestamp,
            last_seen_timestamp,
        }
    }

    pub fn into_json(self) -> Value {
        let mut j = json!({
            "node_key": self.node_key,
            "dgraph.type": "NetworkConnection",
            "src_ip_address": self.src_ip_address,
            "dst_ip_address": self.dst_ip_address,
            "src_port": self.src_port,
            "dst_port": self.dst_port,
        });

        if self.created_timestamp != 0 {
            j["created_timestamp"] = self.created_timestamp.into();
        }

        if self.terminated_timestamp != 0 {
            j["terminated_timestamp"] = self.terminated_timestamp.into();
        }

        if self.last_seen_timestamp != 0 {
            j["last_seen_timestamp"] = self.last_seen_timestamp.into();
        }

        j
    }
}

impl NodeT for NetworkConnection {
    fn get_asset_id(&self) -> Option<&str> {
        None
    }

    fn set_asset_id(&mut self, _asset_id: impl Into<String>) {
        panic!("Can not set asset_id on NetworkConnection");
    }

    fn get_node_key(&self) -> &str {
        &self.node_key
    }

    fn set_node_key(&mut self, node_key: impl Into<String>) {
        self.node_key = node_key.into();
    }

    fn merge(&mut self, other: &Self) -> bool {
        if self.node_key != other.node_key {
            warn!("Attempted to merge two NetworkConnection Nodes with differing node_keys");
            return false;
        }

        let mut merged = false;

        if self.created_timestamp == 0 || other.created_timestamp < self.created_timestamp {
            self.created_timestamp = other.created_timestamp;
            merged = true;
        }
        if self.terminated_timestamp == 0 || other.terminated_timestamp > self.terminated_timestamp
        {
            self.terminated_timestamp = other.terminated_timestamp;
            merged = true;
        }
        if self.last_seen_timestamp == 0 || other.last_seen_timestamp > self.last_seen_timestamp {
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
        mutation_unit.predicate_ref("dgraph.type", MutationPredicateValue::string("NetworkConnection"));
        mutation_unit.predicate_ref("src_ip_address", MutationPredicateValue::string(&self.src_ip_address));
        mutation_unit.predicate_ref("dst_ip_address", MutationPredicateValue::string(&self.dst_ip_address));
        mutation_unit.predicate_ref("src_port", MutationPredicateValue::Number(self.src_port as i64));
        mutation_unit.predicate_ref("dst_port", MutationPredicateValue::Number(self.dst_port as i64));

        if self.created_timestamp != 0 {
            mutation_unit.predicate_ref("created_timestamp", MutationPredicateValue::Number(self.created_timestamp as i64));
        }

        if self.terminated_timestamp != 0 {
            mutation_unit.predicate_ref("terminated_timestamp", MutationPredicateValue::Number(self.terminated_timestamp as i64));
        }

        if self.last_seen_timestamp != 0 {
            mutation_unit.predicate_ref("last_seen_timestamp", MutationPredicateValue::Number(self.last_seen_timestamp as i64));
        }
    }

    fn get_cache_identities_for_predicates(&self) -> Vec<Vec<u8>> {
        let mut predicate_cache_identities = Vec::new();

        predicate_cache_identities.push(format!("{}:{}:{}", self.get_node_key(), "src_ip_address", self.src_ip_address));
        predicate_cache_identities.push(format!("{}:{}:{}", self.get_node_key(), "dst_ip_address", self.dst_ip_address));
        predicate_cache_identities.push(format!("{}:{}:{}", self.get_node_key(), "src_port", self.src_port));
        predicate_cache_identities.push(format!("{}:{}:{}", self.get_node_key(), "dst_port", self.dst_port));

        if self.created_timestamp != 0 {
            predicate_cache_identities.push(format!("{}:{}:{}", self.get_node_key(), "created_timestamp", self.created_timestamp));
        }

        if self.terminated_timestamp != 0 {
            predicate_cache_identities.push(format!("{}:{}:{}", self.get_node_key(), "terminated_timestamp", self.terminated_timestamp));
        }

        if self.last_seen_timestamp != 0 {
            predicate_cache_identities.push(format!("{}:{}:{}", self.get_node_key(), "last_seen_timestamp", self.last_seen_timestamp));
        }

        predicate_cache_identities.into_iter().map(|item| item.as_bytes().to_vec()).collect()
    }
}
