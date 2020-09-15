use std::panic;

use crate::graph_description::*;

pub mod graph_description {
    use derive_builder::*;

    include!(concat!(env!("OUT_DIR"), "/graph_description.rs"));
}

pub mod asset;
pub mod dynamic_node;
pub mod error;
pub mod file;
pub mod graph;
pub mod ip_address;
pub mod ip_connection;
pub mod ip_port;
pub mod network_connection;
pub mod node;
pub mod process;
pub mod process_inbound_connection;
pub mod process_outbound_connection;

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

impl From<String> for NodeProperty {
    fn from(s: String) -> NodeProperty {
        NodeProperty {
            property: Some(node_property::Property::Strprop(s)),
        }
    }
}

impl From<i64> for NodeProperty {
    fn from(i: i64) -> NodeProperty {
        NodeProperty {
            property: Some(node_property::Property::Intprop(i)),
        }
    }
}

impl From<u64> for NodeProperty {
    fn from(i: u64) -> NodeProperty {
        NodeProperty {
            property: Some(node_property::Property::Uintprop(i)),
        }
    }
}

impl std::string::ToString for NodeProperty {
    fn to_string(&self) -> String {
        let prop = match &self.property {
            Some(node_property::Property::Intprop(i)) => i.to_string(),
            Some(node_property::Property::Uintprop(i)) => i.to_string(),
            Some(node_property::Property::Strprop(s)) => s.to_string(),
            None => panic!("Invalid property : {:?}", self),
        };
        prop
    }
}

impl NodeProperty {
    pub fn as_str_prop(&self) -> Option<&str> {
        match &self.property {
            Some(node_property::Property::Strprop(s)) => Some(s),
            _ => None,
        }
    }

    pub fn as_uint_prop(&self) -> Option<u64> {
        match &self.property {
            Some(node_property::Property::Uintprop(s)) => Some(*s),
            _ => None,
        }
    }

    pub fn as_int_prop(&self) -> Option<i64> {
        match &self.property {
            Some(node_property::Property::Intprop(s)) => Some(*s),
            _ => None,
        }
    }
}
