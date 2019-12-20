extern crate base64;
#[macro_use]
extern crate derive_builder;
#[macro_use]
extern crate log;
extern crate prost;
#[macro_use]
extern crate prost_derive;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate uuid;

extern crate thiserror;

use graph_description::*;

pub mod graph_description {
    include!(concat!(env!("OUT_DIR"), "/graph_description.rs"));
}

pub mod error;
pub mod node;
pub mod process;
pub mod file;
pub mod asset;
pub mod ip_address;
pub mod ip_port;
pub mod ip_connection;
pub mod network_connection;
pub mod process_outbound_connection;
pub mod process_inbound_connection;
pub mod dynamic_node;
pub mod graph;


impl From<Static> for IdStrategy {
    fn from(strategy: Static) -> IdStrategy {
        IdStrategy {
            strategy: Some(id_strategy::Strategy::Static(strategy))
        }
    }
}

impl From<Session> for IdStrategy {
    fn from(strategy: Session) -> IdStrategy {
        IdStrategy {
            strategy: Some(id_strategy::Strategy::Session(strategy))
        }
    }
}

impl From<String> for NodeProperty {
    fn from(s: String) -> NodeProperty {
        NodeProperty {
            property: Some(node_property::Property::Strprop(s))
        }
    }
}

impl From<i64> for NodeProperty {
    fn from(i: i64) -> NodeProperty {
        NodeProperty {
            property: Some(node_property::Property::Intprop(i))
        }
    }
}

impl From<u64> for NodeProperty {
    fn from(i: u64) -> NodeProperty {
        NodeProperty {
            property: Some(node_property::Property::Uintprop(i))
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

//
//#[cfg(test)]
//mod tests {
//    use super::*;
//    #[test]
//    fn it_works() {
//        assert_eq!(2 + 2, 4);
//    }
//}
//
//
