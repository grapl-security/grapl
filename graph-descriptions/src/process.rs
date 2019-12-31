use error::Error;
use uuid::Uuid;
use graph_description::Process;
use serde_json::Value;
use std::convert::TryFrom;
use node::NodeT;


#[derive(Debug, Clone)]
pub enum ProcessState {
    Created,
    Terminated,
    Existing
}

impl From<ProcessState> for u32 {
    fn from(p: ProcessState) -> u32 {
        match p {
            ProcessState::Created => 1,
            ProcessState::Terminated => 2,
            ProcessState::Existing => 3,
        }
    }
}

impl TryFrom<u32> for ProcessState {
    type Error = Error;

    fn try_from(p: u32) -> Result<ProcessState, Error> {
        match p {
            1 => Ok(ProcessState::Created),
            2 => Ok(ProcessState::Terminated),
            3 => Ok(ProcessState::Existing),
            _ => Err(Error::InvalidProcessState(p))
        }
    }
}

impl Process {
    pub fn new(asset_id: impl Into<Option<String>>,
               hostname: impl Into<Option<String>>,
               state: ProcessState,
               process_id: u64,
               timestamp: u64,
               process_name: String,
               operating_system: String,
               process_command_line: String,
               process_guid: String,
    ) -> Process {
        let asset_id = asset_id.into();
        let hostname = hostname.into();

        if asset_id.is_none() && hostname.is_none() {
            panic!("AssetID or Hostname must be provided for ProcessOutboundConnection");
        }

        let mut pd = Self {
            node_key: Uuid::new_v4().to_string(),
            asset_id: asset_id.into(),
            hostname: hostname.into(),
            state: state.clone().into(),
            process_id,
            process_name,
            created_timestamp: 0,
            terminated_timestamp: 0,
            last_seen_timestamp: 0,
            operating_system,
            process_command_line,
            process_guid,
        };

        match state {
            ProcessState::Created => pd.created_timestamp = timestamp,
            ProcessState::Existing => pd.last_seen_timestamp = timestamp,
            ProcessState::Terminated => pd.terminated_timestamp = timestamp,
        }

        pd
    }

    pub fn into_json(self) -> Value {
        let asset_id = self.asset_id.as_ref().unwrap();

        let mut j =
            json!({
            "node_key": self.node_key,
            "asset_id": asset_id,
            "process_id": self.process_id,
            "dgraph.type": "Process"
        });

        if !self.process_name.is_empty() {
            j["process_name"] = Value::from(self.process_name);
        }

        if !self.operating_system.is_empty() {
            j["operating_system"] = Value::from(self.operating_system);
        }

        if !self.process_command_line.is_empty() {
            j["process_command_line"] = Value::from(self.process_command_line);
        }

        if !self.process_guid.is_empty() {
            j["process_guid"] = Value::from(self.process_guid);
        }

        if self.created_timestamp!= 0 {
            j["created_timestamp"] = self.created_timestamp.into()
        }

        if self.terminated_timestamp != 0 {
            j["terminated_timestamp"] = self.terminated_timestamp.into()
        }
        if self.last_seen_timestamp != 0 {
            j["last_seen_timestamp"] = self.last_seen_timestamp.into()
        }

        j
    }
}

impl NodeT for Process {
    fn get_asset_id(&self) -> Option<&str> {
        self.asset_id.as_ref().map(String::as_str)
    }

    fn set_asset_id(&mut self, asset_id: impl Into<String>) {
        self.asset_id = Some(asset_id.into())
    }

    fn get_node_key(&self) -> &str {
        self.node_key.as_str()
    }

    fn set_node_key(&mut self, node_key: impl Into<String>) {
        self.node_key = node_key.into();
    }

    fn merge(&mut self, other: &Self) -> bool {
        if self.node_key != other.node_key {
            warn!("Attempted to merge two Process Nodes with differing node_keys");
            return false
        }

        let mut merged = false;

        if self.process_name.is_empty() && !other.process_name.is_empty() {
            self.process_name = other.process_name.clone();
            merged = true;
        }

        if self.operating_system.is_empty() && !other.operating_system.is_empty() {
            self.operating_system = other.operating_system.clone();
            merged = true;
        }

        if self.process_command_line.is_empty() && !other.process_command_line.is_empty() {
            self.process_command_line = other.process_command_line.clone();
            merged = true;
        }

        if self.process_guid.is_empty() && !other.process_guid.is_empty() {
            self.process_guid = other.process_guid.clone();
            merged = true;
        }

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
        if self.node_key != other.node_key {
            warn!("Attempted to merge two IpPort Nodes with differing node_keys");
            return false
        }

        let mut merged = false;

        if self.process_name.is_empty() {
            self.process_name = other.process_name;
            merged = true;
        }

        if self.operating_system.is_empty() {
            self.operating_system = other.operating_system;
            merged = true;
        }

        if self.process_command_line.is_empty() {
            self.process_command_line = other.process_command_line;
            merged = true;
        }

        if self.process_guid.is_empty() && !other.process_guid.is_empty() {
            self.process_guid = other.process_guid;
            merged = true;
        }

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
}