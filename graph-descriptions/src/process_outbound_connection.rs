use error::Error;
use uuid::Uuid;
use graph_description::ProcessOutboundConnection;
use serde_json::Value;
use std::convert::TryFrom;
use node::NodeT;


pub enum ProcessOutboundConnectionState {
    Connected,
    Existing,
    Closed,
}

impl From<ProcessOutboundConnectionState> for u32 {
    fn from(p: ProcessOutboundConnectionState) -> u32 {
        match p {
            ProcessOutboundConnectionState::Connected => 1,
            ProcessOutboundConnectionState::Closed => 2,
            ProcessOutboundConnectionState::Existing => 3,
        }
    }
}

impl TryFrom<u32> for ProcessOutboundConnectionState {
    type Error = Error;

    fn try_from(p: u32) -> Result<ProcessOutboundConnectionState, Error> {
        match p {
            1 => Ok(ProcessOutboundConnectionState::Connected),
            2 => Ok(ProcessOutboundConnectionState::Closed),
            3 => Ok(ProcessOutboundConnectionState::Existing),
            _ => Err(Error::InvalidProcessOutboundConnectionState(p))
        }
    }
}

impl ProcessOutboundConnection {
    pub fn new(
        asset_id: impl Into<Option<String>>,
        hostname: impl Into<Option<String>>,
        state: ProcessOutboundConnectionState,
        port: u16,
        ip_address: impl Into<String>,
        protocol: impl Into<String>,
        created_timestamp: u64,
        terminated_timestamp: u64,
        last_seen_timestamp: u64,
    ) -> Self {
        let asset_id = asset_id.into();
        let hostname = hostname.into();
        let protocol = protocol.into();

        if hostname.is_none() && asset_id.is_none() {
            panic!("ProcessOutboundConnection must have at least asset_id or hostname");
        }

        let ip_address = ip_address.into();

        Self {
            node_key: Uuid::new_v4().to_string(),
            ip_address,
            asset_id,
            hostname,
            protocol,
            created_timestamp,
            terminated_timestamp,
            last_seen_timestamp,
            port: port as u32,
            state: state.into(),
        }
    }

    pub fn into_json(self) -> Value {
        let mut j = json!({
            "node_key": self.node_key,
            "dgraph.type": "ProcessOutboundConnection",
            "asset_id": self.asset_id.unwrap(),
            "protocol": self.protocol,
            "port": self.port,
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

impl NodeT for ProcessOutboundConnection {
    fn get_asset_id(&self) -> Option<&str> {
        self.asset_id.as_ref().map(String::as_str)
    }

    fn set_asset_id(&mut self, asset_id: impl Into<String>) {
        self.asset_id = Some(asset_id.into());
    }

    fn get_node_key(&self) -> &str {
        &self.node_key
    }

    fn set_node_key(&mut self, node_key: impl Into<String>) {
        self.node_key = node_key.into();
    }

    fn merge(&mut self, other: &Self) -> bool {
        if self.node_key != other.node_key {
            warn!("Attempted to merge two ProcessOutboundConnection Nodes with differing node_keys");
            return false
        }

        if self.ip_address != other.ip_address {
            warn!("Attempted to merge two ProcessOutboundConnection Nodes with differing IPs");
            return false;
        }

        let mut merged = false;

        if self.asset_id.is_none() && other.asset_id.is_some() {
            self.asset_id = other.asset_id.clone();
        }

        if self.hostname.is_none() && other.hostname.is_some() {
            self.hostname = other.hostname.clone();
        }

        if self.created_timestamp != 0 && self.created_timestamp > other.created_timestamp {
            self.created_timestamp = other.created_timestamp;
            merged = true;
        }

        if self.terminated_timestamp != 0 && self.terminated_timestamp < other.terminated_timestamp {
            self.terminated_timestamp = other.terminated_timestamp;
            merged = true;
        }

        if self.last_seen_timestamp != 0 && self.last_seen_timestamp < other.last_seen_timestamp {
            self.last_seen_timestamp = other.last_seen_timestamp;
            merged = true;
        }

        merged
    }

    fn merge_into(&mut self, other: Self) -> bool {
        self.merge(&other)
    }
}