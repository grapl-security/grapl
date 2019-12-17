use uuid::Uuid;
use graph_description::Asset;
use serde_json::Value;
use node::NodeT;


impl Asset {
    pub fn new(asset_id: impl Into<Option<String>>,
               hostname: impl Into<Option<String>>,
               mac_address: impl Into<Option<String>>,
               first_seen_timestamp: u64,
               last_seen_timestamp: u64,
    ) -> Self {
        let asset_id = asset_id.into();
        let hostname = hostname.into();

        if asset_id.is_none() && hostname.is_none() {
            panic!("AssetID or Hostname must be provided for ProcessOutboundConnection");
        }

        Self {
            node_key: Uuid::new_v4().to_string(),
            asset_id,
            hostname,
            mac_address: mac_address.into(),
            first_seen_timestamp,
            last_seen_timestamp,
        }
    }

    pub fn into_json(self) -> Value {
        let asset_id = self.asset_id.as_ref().unwrap();
        let mut j = json!({
            "node_key": self.node_key,
            "asset_id": asset_id,
            "dgraph.type": "Asset",
        });

        if self.first_seen_timestamp != 0 {
            j["first_seen_timestamp"] = self.first_seen_timestamp.into();
        }

        if self.last_seen_timestamp != 0 {
            j["last_seen_timestamp"] = self.last_seen_timestamp.into();
        }

        if let Some(hostname) = self.hostname {
            j["hostname"] = Value::from(hostname.clone());
        }

        if let Some(mac_address) = self.mac_address {
            j["mac_address"] = Value::from(mac_address.clone());
        }

        j
    }
}

impl NodeT for Asset {
    fn get_asset_id(&self) -> Option<&str> {
        self.asset_id.as_ref().map(|asset_id| asset_id.as_str())
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
            warn!("Attempted to merge two Asset Nodes with differing node_keys");
            return false
        }

        let mut merged = false;

        if self.asset_id.is_none() && other.asset_id.is_some() {
            merged = true;
            self.asset_id = other.asset_id.clone();
        }

        if self.hostname.is_none() && other.hostname.is_some() {
            merged = true;
            self.hostname = other.hostname.clone();
        }

        if self.mac_address.is_none() && other.mac_address.is_some() {
            merged = true;
            self.mac_address = other.mac_address.clone();
        }

        merged
    }

    fn merge_into(&mut self, other: Self) -> bool {
        if self.node_key != other.node_key {
            warn!("Attempted to merge two Asset Nodes with differing node_keys");
            return false
        }

        let mut merged = false;

        if self.asset_id.is_none() && other.asset_id.is_some() {
            self.asset_id = other.asset_id;
            merged = true;
        }

        if self.hostname.is_none() && other.hostname.is_some() {
            self.hostname = other.hostname;
            merged = true;
        }

        if self.mac_address.is_none() && other.mac_address.is_some() {
            self.mac_address = other.mac_address;
            merged = true;
        }

        if other.first_seen_timestamp != 0 && self.first_seen_timestamp > other.first_seen_timestamp {
            self.first_seen_timestamp = other.first_seen_timestamp;
            merged = true;
        }

        if other.last_seen_timestamp != 0 && self.last_seen_timestamp < other.last_seen_timestamp {
            self.last_seen_timestamp = other.last_seen_timestamp;
            merged = true;
        }

        merged
    }
}