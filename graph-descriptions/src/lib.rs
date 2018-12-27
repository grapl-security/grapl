extern crate base64;
#[macro_use] extern crate custom_derive;
#[macro_use]
extern crate derive_builder;
#[macro_use]
extern crate derive_more;
extern crate hash_hasher;
#[macro_use]
extern crate log;
#[macro_use]
extern crate maplit;
#[macro_use] extern crate newtype_derive;
extern crate prost;
#[macro_use]
extern crate prost_derive;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate sha3;
extern crate uuid;

use std::collections::HashMap;

use graph_description::*;
use graph_description::host::HostId;
use graph_description::node_description::*;
use hash_hasher::HashBuildHasher;
use serde_json::Value;
use sha3::Digest;
use sha3::Keccak256;
use uuid::Uuid;

pub mod graph_description {
    include!(concat!(env!("OUT_DIR"), "/graph_description.rs"));
}


impl GeneratedSubgraphs {
    pub fn new(subgraphs: Vec<GraphDescription>) -> GeneratedSubgraphs {
        GeneratedSubgraphs {
            subgraphs
        }
    }
}

#[derive(Debug, Clone)]
pub enum ConnectionState {
    Created,
    Terminated,
    Existing,
}


impl Into<u32> for ConnectionState {
    fn into(self) -> u32 {
        match self {
            ConnectionState::Created => 1,
            ConnectionState::Terminated => 2,
            ConnectionState::Existing => 3,
        }
    }
}


impl OutboundConnection {
    pub fn new(
        asset_id: impl Into<Option<String>>,
        hostname: impl Into<Option<String>>,
        host_ip: impl Into<Option<Vec<u8>>>,
        state: ConnectionState,
        port: u32,
        timestamp: u64,
    ) -> OutboundConnection {
        let mut ic = Self {
            node_key: Uuid::new_v4().to_string(),
            asset_id: asset_id.into(),
            hostname: hostname.into(),
            host_ip: host_ip.into(),
            state: state.clone().into(),
            port,
            created_timestamp: 0,
            terminated_timestamp: 0,
            last_seen_timestamp: 0,
        };

        match state {
            ConnectionState::Created => ic.created_timestamp= timestamp,
            ConnectionState::Terminated => ic.terminated_timestamp = timestamp,
            ConnectionState::Existing => ic.last_seen_timestamp = timestamp,
        }
        ic
    }

    pub fn clone_key(&self) -> String {
        self.node_key.clone()
    }

    pub fn into_json(self) -> Value {
        let asset_id = self.asset_id.as_ref().unwrap();

        let mut j = json!({
            "node_key": self.node_key,
            "asset_id": asset_id,
            "port": self.port,
            "direction": "inbound",
        });

        if self.created_timestamp!= 0 {
            j["created_time"] = self.created_timestamp.into()
        }

        if self.terminated_timestamp != 0 {
            j["terminated_timestamp"] = self.terminated_timestamp.into()
        }
        if self.last_seen_timestamp != 0 {
            j["last_seen_timestamp"] = self.last_seen_timestamp.into()
        }

        j
    }

    pub fn set_key(&mut self, key: String) {
        self.node_key = key;
    }

    pub fn set_asset_id(&mut self, asset_id: String) {
        self.asset_id = Some(asset_id)
    }

    pub fn merge(&mut self, other: &Self) {
        if self.node_key != other.node_key {
            warn!("Attempted to merge two nodes with different keys. Dropping merge.");
            return
        }

        if self.created_timestamp == 0 {
            self.created_timestamp = other.created_timestamp;
        }
        if self.terminated_timestamp == 0 {
            self.terminated_timestamp = other.terminated_timestamp;
        }
        if self.last_seen_timestamp == 0 {
            self.last_seen_timestamp = other.last_seen_timestamp;
        }
    }

    pub fn timestamp(&self) -> u64 {
        match ConnectionState::from(self.state) {
            ConnectionState::Created => self.created_timestamp,
            ConnectionState::Terminated => self.terminated_timestamp,
            ConnectionState::Existing => self.last_seen_timestamp,
        }
    }
}


impl InboundConnection {
    pub fn new(
        asset_id: impl Into<Option<String>>,
        hostname: impl Into<Option<String>>,
        host_ip: impl Into<Option<Vec<u8>>>,
        state: ConnectionState,
        port: u32,
        timestamp: u64,
    ) -> InboundConnection {
        let mut ic = Self {
            node_key: Uuid::new_v4().to_string(),
            asset_id: asset_id.into(),
            hostname: hostname.into(),
            host_ip: host_ip.into(),
            state: state.clone().into(),
            port,
            created_timestamp: 0,
            terminated_timestamp: 0,
            last_seen_timestamp: 0,
        };

        match state {
            ConnectionState::Created => ic.created_timestamp= timestamp,
            ConnectionState::Terminated => ic.terminated_timestamp = timestamp,
            ConnectionState::Existing => ic.last_seen_timestamp = timestamp,
        }
        ic
    }

    pub fn clone_key(&self) -> String {
        self.node_key.clone()
    }

    pub fn get_key(&self) -> &str {
        &self.node_key
    }

    pub fn into_json(self) -> Value {
        let asset_id = self.asset_id.as_ref().unwrap();

        let mut j = json!({
            "node_key": self.node_key,
            "asset_id": asset_id,
            "port": self.port,
            "direction": "inbound",
        });

        if self.created_timestamp!= 0 {
            j["created_time"] = self.created_timestamp.into()
        }

        if self.terminated_timestamp != 0 {
            j["terminated_timestamp"] = self.terminated_timestamp.into()
        }
        if self.last_seen_timestamp != 0 {
            j["last_seen_timestamp"] = self.last_seen_timestamp.into()
        }

        j
    }

    pub fn set_key(&mut self, key: String) {
        self.node_key = key;
    }

    pub fn set_asset_id(&mut self, asset_id: String) {
        self.asset_id = Some(asset_id)
    }

    pub fn merge(&mut self, other: &Self) {
        if self.node_key != other.node_key {
            warn!("Attempted to merge two nodes with different keys. Dropping merge.");
            return
        }

        if self.created_timestamp == 0 {
            self.created_timestamp = other.created_timestamp;
        }
        if self.terminated_timestamp == 0 {
            self.terminated_timestamp = other.terminated_timestamp;
        }
        if self.last_seen_timestamp == 0 {
            self.last_seen_timestamp = other.last_seen_timestamp;
        }
    }

    pub fn timestamp(&self) -> u64 {
        match ConnectionState::from(self.state) {
            ConnectionState::Created => self.created_timestamp,
            ConnectionState::Terminated => self.terminated_timestamp,
            ConnectionState::Existing => self.last_seen_timestamp,
        }
    }
}

enum IntoEdge {
    //Process -> Process Edges
    CreatedProcess,
    //Process -> File Edges
    CreatedFile,
    DeletedFile,
    ExecutedFromFile,
    ReadFromFile,
    WroteToFile,
}

macro_rules! node_from {
    ($t: ident, $n: ident) => (
        impl From<$t> for NodeDescription {
            fn from(t: $t) -> Self {
                NodeDescription {
                    which_node: WhichNode::$n(
                        t
                    ).into()
                }
            }
        }
    )
}

node_from!(IpAddressDescription, IpAddressNode);
node_from!(ProcessDescription, ProcessNode);
node_from!(FileDescription, FileNode);
node_from!(OutboundConnection, OutboundConnectionNode);
node_from!(InboundConnection, InboundConnectionNode);



#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum HostIdentifier {
    IpAddress(Vec<u8>),
    Hostname(String),
    AssetId(String),
}

impl From<HostId> for HostIdentifier {
    fn from(host_id: HostId) -> Self {
        match host_id {
            HostId::Hostname(hostname) => HostIdentifier::Hostname(hostname),
            HostId::Ip(hostname) => HostIdentifier::IpAddress(hostname),
            HostId::AssetId(hostname) => HostIdentifier::AssetId(hostname),
        }
    }
}

impl From<HostIdentifier> for HostId {
    fn from(host_id: HostIdentifier) -> Self {
        match host_id {
            HostIdentifier::Hostname(hostname) => HostId::Hostname(hostname),
            HostIdentifier::IpAddress(hostname) => HostId::Ip(hostname),
            HostIdentifier::AssetId(hostname) => HostId::AssetId(hostname),
        }
    }
}

impl From<HostIdentifier> for Host {
    fn from(host_id: HostIdentifier) -> Self {
        match host_id {
            HostIdentifier::Hostname(hostname) => {
                Host {
                    host_id: Some(
                        HostId::Hostname(hostname)
                    )
                }
            },
            HostIdentifier::IpAddress(hostname) => {
                Host {
                    host_id: Some(
                        HostId::Ip(hostname)
                    )
                }
            }
            HostIdentifier::AssetId(hostname) => {
                Host {
                    host_id: Some(
                        HostId::AssetId(hostname)
                    )
                }
            }
        }
    }
}

impl HostIdentifier {
    pub fn as_asset_id(&self) -> Option<&str> {
        match self {
            HostIdentifier::AssetId(asset_id) => Some(asset_id.as_ref()),
            _ => None
        }
    }
}



#[derive(Clone, Debug)]
pub enum Node {
    ProcessNode(ProcessDescription),
    FileNode(FileDescription),
    IpAddressNode(IpAddressDescription),
    OutboundConnectionNode(OutboundConnection),
    InboundConnectionNode(InboundConnection),
}

impl NodeDescription {
    pub fn which(self) -> Node {
        match self.which_node.clone().unwrap() {
            WhichNode::ProcessNode(n) => Node::ProcessNode(n.into()),
            WhichNode::FileNode(n) => Node::FileNode(n.into()),
            WhichNode::IpAddressNode(n) => Node::IpAddressNode(n.into()),
            WhichNode::OutboundConnectionNode(n) => Node::OutboundConnectionNode(n.into()),
            WhichNode::InboundConnectionNode(n) => Node::InboundConnectionNode(n.into()),
        }
    }

    pub fn get_key(&self) -> &str {
        match self.which_node.as_ref().unwrap() {
            WhichNode::ProcessNode(n) => n.node_key.as_ref(),
            WhichNode::FileNode(n) => n.node_key.as_ref(),
            WhichNode::IpAddressNode(n) => n.node_key.as_ref(),
            WhichNode::OutboundConnectionNode(n) => n.node_key.as_ref(),
            WhichNode::InboundConnectionNode(n) => n.node_key.as_ref(),
        }
    }

    pub fn get_timestamp(&self) -> u64 {
        match self.which_node.as_ref().unwrap() {
            WhichNode::ProcessNode(ref node) => {
                match ProcessState::from(node.state) {
                    ProcessState::Created => node.created_timestamp,
                    ProcessState::Terminated => node.terminated_timestamp,
                    ProcessState::Existing => node.last_seen_timestamp,
                }
            }
            WhichNode::FileNode(ref node) => {
                match FileState::from(node.state) {
                    FileState::Created => node.created_timestamp,
                    FileState::Deleted => node.deleted_timestamp,
                    FileState::Existing => node.last_seen_timestamp,
                }
            }
            WhichNode::IpAddressNode(ref node) => {
                node.timestamp
            }
            WhichNode::OutboundConnectionNode(ref node) => {
                match ConnectionState::from(node.state) {
                    ConnectionState::Created => node.created_timestamp,
                    ConnectionState::Terminated => node.terminated_timestamp,
                    ConnectionState::Existing => node.last_seen_timestamp,
                }
            }
            WhichNode::InboundConnectionNode(ref node) => {
                match ConnectionState::from(node.state) {
                    ConnectionState::Created => node.created_timestamp,
                    ConnectionState::Terminated => node.terminated_timestamp,
                    ConnectionState::Existing => node.last_seen_timestamp,
                }
            }
        }
    }

    pub fn set_asset_id(&mut self, asset_id: String) {
        match self.which_node.as_mut().unwrap() {
            WhichNode::ProcessNode(ref mut node) => {
                node.asset_id = Some(asset_id)
            }
            WhichNode::FileNode(ref mut node) => {
                node.asset_id = Some(asset_id)
            }
            WhichNode::IpAddressNode(_) => {
                panic!("ip address node has no asset id")
            }
            WhichNode::OutboundConnectionNode(ref mut node) => {
                node.asset_id = Some(asset_id)
            }
            WhichNode::InboundConnectionNode(ref mut node) => {
                node.asset_id = Some(asset_id)
            }
        }
    }

    pub fn set_key(&mut self, key: String) {
        match self.which_node.as_mut().unwrap() {
            WhichNode::ProcessNode(ref mut node) => {
                node.node_key = key;
            }
            WhichNode::FileNode(ref mut node) => {
                node.node_key = key;
            }
            WhichNode::IpAddressNode(ref mut node) => {
                node.node_key = key;
            }
            WhichNode::OutboundConnectionNode(ref mut node) => {
                node.node_key = key;
            }
            WhichNode::InboundConnectionNode(ref mut node) => {
                node.node_key = key;
            }
        }
    }

    pub fn into_json(self) -> Value {
        match self.which_node.unwrap() {
            WhichNode::ProcessNode(node) => {
                let node: ProcessDescription = node.into();
                node.into_json()
            }
            WhichNode::FileNode(node) => {
                let node: FileDescription = node.into();
                node.into_json()
            }
            WhichNode::IpAddressNode(node) => {
                let node: IpAddressDescription = node.into();
                node.into_json()
            }
            WhichNode::OutboundConnectionNode(node) => {
                let node: OutboundConnection = node.into();
                node.into_json()
            }
            WhichNode::InboundConnectionNode(node) => {
                let node: InboundConnection = node.into();
                node.into_json()
            }
        }
    }

    pub fn merge(&mut self, other: &Self) {
        match (self.which_node.as_mut().unwrap(), other.which_node.as_ref().unwrap()) {
            (WhichNode::ProcessNode(node),              WhichNode::ProcessNode(other)) => node.merge(other),
            (WhichNode::FileNode(node),                 WhichNode::FileNode(other)) => node.merge(other),
            (WhichNode::IpAddressNode(node),            WhichNode::IpAddressNode(other)) => node.merge(other),
            (WhichNode::OutboundConnectionNode(node),   WhichNode::OutboundConnectionNode(other)) => node.merge(other),
            (WhichNode::InboundConnectionNode(node),    WhichNode::InboundConnectionNode(other)) => node.merge(other),

            _ => warn!("Attempted to merge two nodes of different type"),
        }
    }

}


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

impl From<u32> for ProcessState {
    fn from(i: u32) -> ProcessState {
        match i {
            1 => ProcessState::Created,
            2 => ProcessState::Terminated,
            3 => ProcessState::Existing,
            _ => panic!("invalid conversion to process state")
        }
    }
}

impl From<u32> for ConnectionState {
    fn from(i: u32) -> ConnectionState {
        match i {
            1 => ConnectionState::Created,
            2 => ConnectionState::Terminated,
            3 => ConnectionState::Existing,
            _ => panic!("invalid conversion to connection state")
        }
    }
}

#[derive(Debug, Clone)]
pub enum FileState {
    Created,
    Deleted,
    Existing
}

impl From<u32> for FileState {
    fn from(i: u32) -> FileState {
        match i {
            1 => FileState::Created,
            2 => FileState::Deleted,
            3 => FileState::Existing,
            _ => panic!("invalid conversion to file state")
        }
    }
}

impl From<FileState> for u32 {
    fn from(p: FileState) -> u32 {
        match p {
            FileState::Created => 1,
            FileState::Deleted => 2,
            FileState::Existing => 3,
        }

    }
}

impl ProcessDescription {
    pub fn new(asset_id: impl Into<Option<String>>,
               hostname: impl Into<Option<String>>,
               host_ip: impl Into<Option<Vec<u8>>>,
               state: ProcessState,
               pid: u64,
               timestamp: u64,
               image_name: Vec<u8>,
               image_path: Vec<u8>
    ) -> ProcessDescription {
        let mut pd = Self {
            node_key: Uuid::new_v4().to_string(),
            asset_id: asset_id.into(),
            hostname: hostname.into(),
            host_ip: host_ip.into(),
            state: state.clone().into(),
            pid,
            image_name,
            image_path,
            created_timestamp: 0,
            terminated_timestamp: 0,
            last_seen_timestamp: 0,
        };

        match state {
            ProcessState::Created => pd.created_timestamp= timestamp,
            ProcessState::Existing => pd.last_seen_timestamp = timestamp,
            ProcessState::Terminated => pd.terminated_timestamp = timestamp,
        }

        pd
    }

    pub fn get_key(&self) -> &str {
        &self.node_key
    }

    pub fn set_key(&mut self, key: String) {
        self.node_key = key;
    }

    pub fn set_asset_id(&mut self, asset_id: String) {
        self.asset_id = Some(asset_id)
    }

    pub fn clone_key(&self) -> String {
        self.node_key.clone()
    }

    pub fn into_json(self) -> Value {
        let asset_id = self.asset_id.as_ref().unwrap();

        let mut j =
            json!({
            "node_key": self.node_key,
            "asset_id": asset_id,
            "pid": self.pid,

        });

        if !self.image_name.is_empty() {
            j["image_name"] = Value::from(String::from_utf8_lossy(&self.image_name));
        }

        match ProcessState::from(self.state) {
            ProcessState::Created => j["created_time"] = self.created_timestamp.into(),
            ProcessState::Terminated => j["terminated_timestamp"] = self.terminated_timestamp.into(),
            ProcessState::Existing => j["last_seen_timestamp"] = self.last_seen_timestamp.into(),
        }

        j
    }


    pub fn merge(&mut self, other: &Self) {
        if self.node_key != other.node_key {
            warn!("Attempted to merge two process nodes with different keys. Dropping merge.");
            return
        }

        if self.created_timestamp == 0 {
            self.created_timestamp = other.created_timestamp;
        }
        if self.terminated_timestamp == 0 {
            self.terminated_timestamp = other.terminated_timestamp;
        }
        if self.last_seen_timestamp == 0 {
            self.last_seen_timestamp = other.last_seen_timestamp;
        }

        if self.image_name.is_empty() && !other.image_name.is_empty() {
            self.image_name = other.image_name.clone();
        }

        if self.image_path.is_empty() && !other.image_path.is_empty() {
            self.image_path = other.image_path.clone();
        }
    }

    pub fn timestamp(&self) -> u64 {
        match ProcessState::from(self.state) {
            ProcessState::Created => self.created_timestamp,
            ProcessState::Terminated => self.terminated_timestamp,
            ProcessState::Existing => self.last_seen_timestamp,
        }
    }
}

impl OutboundConnection {
    pub fn asset_id(&self) -> &str {
        self.asset_id.as_ref().unwrap()
    }
}

impl InboundConnection {
    pub fn asset_id(&self) -> &str {
        self.asset_id.as_ref().unwrap()
    }
}

impl ProcessDescription {
    pub fn asset_id(&self) -> &str {
        self.asset_id.as_ref().unwrap()
    }
}

impl FileDescription {
    pub fn asset_id(&self) -> &str {
        self.asset_id.as_ref().unwrap()
    }
}

impl FileDescription {
    pub fn new(asset_id: impl Into<Option<String>>,
               hostname: impl Into<Option<String>>,
               host_ip: impl Into<Option<Vec<u8>>>,
               state: FileState,
               timestamp: u64,
               path: Vec<u8>,
    ) -> FileDescription {
        let mut fd = FileDescription {
            node_key: Uuid::new_v4().to_string(),
            asset_id: asset_id.into(),
            hostname: hostname.into(),
            host_ip: host_ip.into(),
            state: state.clone().into(),
            path,
            created_timestamp: 0,
            deleted_timestamp: 0,
            last_seen_timestamp: 0,
        };

        match state {
            FileState::Created => fd.created_timestamp= timestamp,
            FileState::Existing => fd.last_seen_timestamp = timestamp,
            FileState::Deleted => fd.deleted_timestamp = timestamp,
        }

        fd
    }

    pub fn get_key(&self) -> &str {
        &self.node_key
    }

    pub fn set_key(&mut self, key: String) {
        self.node_key = key;
    }

    pub fn set_asset_id(&mut self, asset_id: String) {
        self.asset_id = Some(asset_id)
    }

    pub fn clone_key(&self) -> String {
        self.node_key.clone()
    }

    pub fn into_json(self) -> Value {
        let asset_id = self.asset_id.as_ref().unwrap();
        let mut j = json!({
            "node_key": self.node_key,
            "asset_id": asset_id,
        });

        if !self.path.is_empty() {
            j["path"] = Value::from(String::from_utf8_lossy(&self.path));
        }

        if self.created_timestamp!= 0 {
            j["created_time"] = self.created_timestamp.into()
        }

        if self.deleted_timestamp != 0 {
            j["deleted_timestamp"] = self.deleted_timestamp.into()
        }
        if self.last_seen_timestamp != 0 {
            j["last_seen_timestamp"] = self.last_seen_timestamp.into()
        }

        j
    }

    pub fn merge(&mut self, other: &Self) {
        if self.node_key != other.node_key {
            warn!("Attempted to merge two process nodes with different keys. Dropping merge.");
            return
        }

        if self.created_timestamp == 0 {
            self.created_timestamp = other.created_timestamp;
        }
        if self.deleted_timestamp == 0 {
            self.deleted_timestamp = other.deleted_timestamp;
        }
        if self.last_seen_timestamp == 0 {
            self.last_seen_timestamp = other.last_seen_timestamp;
        }

        if self.path.is_empty() && !other.path.is_empty() {
            self.path = other.path.clone();
        }
    }

    pub fn timestamp(&self) -> u64 {
        match FileState::from(self.state) {
            FileState::Created => self.created_timestamp,
            FileState::Deleted => self.deleted_timestamp,
            FileState::Existing => self.last_seen_timestamp,
        }
    }
}

impl IpAddressDescription {
    pub fn new(timestamp: u64,
               ip_address: Vec<u8>,
    ) -> IpAddressDescription {
        // 20 is based on the max size of a base encoded ipv4 ip
        let mut node_key = String::with_capacity(20);
        base64::encode_config_buf(&ip_address,
                                  base64::STANDARD,
                                  &mut node_key);

        IpAddressDescription {
            node_key,
            timestamp,
            ip_address
        }
    }

    pub fn get_key(&self) -> &str {
        &self.node_key
    }

    pub fn set_key(&mut self, key: String) {
        self.node_key = key;
    }

    pub fn clone_key(&self) -> String {
        self.node_key.clone()
    }


    pub fn into_json(self) -> Value {
        json!({
            "node_key": self.node_key,
            "last_seen": self.timestamp,
            "ip_address": self.ip_address,
        })
    }

    pub fn merge(&mut self, _other: &Self) {
        // nop
    }


}

impl GraphDescription {
    pub fn new(timestamp: u64) -> Self {
        GraphDescription {
            nodes: hashmap![],
            edges: hashmap![],
            timestamp
        }
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty() && self.edges.is_empty()
    }

    pub fn merge(&mut self, other: &GraphDescription) {
        self.edges.extend(other.edges.clone());

        for (node_key, other_node) in other.nodes.iter() {
            self.nodes
                .entry(node_key.clone())
                .and_modify(|node| {
                    node.merge(other_node);
                })
                .or_insert(other_node.clone());
        }
    }

    pub fn add_node<N>(&mut self, node: N)
        where N: Into<NodeDescription>
    {
        let node = node.into();
        let key = node.get_key().to_owned();

        self.nodes.insert(key.clone(), node);
        self.edges
            .entry(key)
            .or_insert_with(|| {
                EdgeList { edges: vec![] }
            });
    }


    pub fn with_node<N>(self, node: N) -> GraphDescription
        where N: Into<NodeDescription>
    {
        let mut _self = self;
        _self.add_node(node);
        _self
    }

    pub fn add_edge(&mut self,
                    edge_name: impl Into<String>,
                    from: impl Into<String>,
                    to: impl Into<String>)
    {
        let from = from.into();
        let to = to.into();
        let edge_name = edge_name.into();
        let edge = EdgeDescription {
            from_neighbor_key: from.clone(),
            to_neighbor_key: to,
            edge_name
        };

        self.edges
            .entry(from)
            .or_insert_with(|| {
                EdgeList { edges: Vec::with_capacity(1) }
            })
            .edges.push(edge);
    }
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}


