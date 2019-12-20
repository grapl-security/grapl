use std::convert::TryFrom;

use serde_json::Value;
use uuid::Uuid;

use error::Error;
use graph_description::IpConnection;
use node::NodeT;

pub enum IpConnectionState {
    Created,
    Existing,
    Terminated,
}

impl From<IpConnectionState> for u32 {
    fn from(p: IpConnectionState) -> u32 {
        match p {
            IpConnectionState::Created => 1,
            IpConnectionState::Terminated => 2,
            IpConnectionState::Existing => 3,
        }
    }
}

impl TryFrom<u32> for IpConnectionState {
    type Error = Error;

    fn try_from(p: u32) -> Result<IpConnectionState, Error> {
        match p {
            1 => Ok(IpConnectionState::Created),
            2 => Ok(IpConnectionState::Terminated),
            3 => Ok(IpConnectionState::Existing),
            _ => Err(Error::InvalidIpConnectionState(p))
        }
    }
}


impl IpConnection {
    pub fn new(
        src_ip_address: impl Into<String>,
        dst_ip_address: impl Into<String>,
        protocol: impl Into<String>,
        state: IpConnectionState,
        created_timestamp: u64,
        terminated_timestamp: u64,
        last_seen_timestamp: u64,
    ) -> Self {
        let src_ip_address= src_ip_address.into();
        let dst_ip_address= dst_ip_address.into();
        let protocol = protocol.into();

        Self {
            node_key: Uuid::new_v4().to_string(),
            src_ip_address,
            dst_ip_address,
            protocol,
            state: state.into(),
            created_timestamp,
            terminated_timestamp,
            last_seen_timestamp,
        }
    }

    pub fn into_json(self) -> Value {
        let mut j = json!({
            "node_key": self.node_key,
            "dgraph.type": "IpConnection",
            "src_ip_address": self.src_ip_address,
            "dst_ip_address": self.dst_ip_address,
            "protocol": self.protocol,
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

impl NodeT for IpConnection {
    fn get_asset_id(&self) -> Option<&str> {
        None
    }

    fn set_asset_id(&mut self, _asset_id: impl Into<String>) {
        panic!("Can not set asset_id on IpConnection");
    }

    fn get_node_key(&self) -> &str {
        &self.node_key
    }

    fn set_node_key(&mut self, node_key: impl Into<String>) {
        self.node_key = node_key.into();
    }

    fn merge(&mut self, other: &Self) -> bool {
        if self.node_key != other.node_key {
            warn!("Attempted to merge two IpConnection Nodes with differing node_keys");
            return false
        }

        let mut merged = false;

        if self.created_timestamp == 0 || other.created_timestamp < self.created_timestamp {
            self.created_timestamp = other.created_timestamp;
            merged = true;
        }
        if self.terminated_timestamp == 0 || other.terminated_timestamp > self.terminated_timestamp {
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
}