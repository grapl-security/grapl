use dgraph_query_lib::mutation::{MutationPredicateValue,
                                 MutationUnit};
use log::warn;
use serde_json::{json,
                 Value};

use crate::{graph_description::IpPort,
            node::NodeT};

impl IpPort {
    pub fn new(ip_address: impl Into<String>, port: u16, protocol: impl Into<String>) -> Self {
        let ip_address = ip_address.into();
        let protocol = protocol.into();

        Self {
            node_key: format!("{}{}{}", ip_address, port, protocol),
            ip_address,
            port: port as u32,
            protocol,
        }
    }

    pub fn into_json(self) -> Value {
        json!({
            "node_key": self.node_key,
            "dgraph.type": "IpPort",
            "port": self.port,
            "protocol": self.protocol,
        })
    }
}

impl NodeT for IpPort {
    fn get_asset_id(&self) -> Option<&str> {
        None
    }

    fn set_asset_id(&mut self, _asset_id: impl Into<String>) {
        panic!("Can not set asset_id on IpPort");
    }

    fn get_node_key(&self) -> &str {
        &self.node_key
    }

    fn set_node_key(&mut self, node_key: impl Into<String>) {
        self.node_key = node_key.into();
    }

    fn merge(&mut self, other: &Self) -> bool {
        if self.node_key != other.node_key {
            warn!("Attempted to merge two IpPort Nodes with differing node_keys");
            return false;
        }

        if self.ip_address != other.ip_address {
            warn!("Attempted to merge two IpPort Nodes with differing IPs");
            return false;
        }

        // There is no variable information in an IpPort
        false
    }

    fn merge_into(&mut self, _other: Self) -> bool {
        false
    }

    fn attach_predicates_to_mutation_unit(&self, mutation_unit: &mut MutationUnit) {
        mutation_unit.predicate_ref("node_key", MutationPredicateValue::string(&self.node_key));
        mutation_unit.predicate_ref("dgraph.type", MutationPredicateValue::string("IpPort"));
        mutation_unit.predicate_ref("protocol", MutationPredicateValue::string(&self.protocol));
        mutation_unit.predicate_ref("port", MutationPredicateValue::Number(self.port as i64));
    }

    fn get_cache_identities_for_predicates(&self) -> Vec<Vec<u8>> {
        let mut predicate_cache_identities = Vec::new();

        predicate_cache_identities.push(format!(
            "{}:{}:{}",
            self.get_node_key(),
            "port",
            self.port
        ));
        predicate_cache_identities.push(format!(
            "{}:{}:{}",
            self.get_node_key(),
            "protocol",
            self.protocol
        ));

        predicate_cache_identities
            .into_iter()
            .map(|item| item.as_bytes().to_vec())
            .collect()
    }
}
