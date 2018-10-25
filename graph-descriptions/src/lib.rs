#![feature(nll, stdsimd)]
extern crate base64;
#[macro_use] extern crate custom_derive;
#[macro_use]
extern crate derive_more;
extern crate hash_hasher;
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
#[macro_use]
extern crate typed_builder;
extern crate uuid;

use graph_description::*;
use graph_description::host::HostId;
use graph_description::node_description_proto::*;
use hash_hasher::HashBuildHasher;
use serde_json::Value;
use sha3::Digest;
use sha3::Keccak256;
use std::collections::HashMap;
use uuid::Uuid;

pub mod graph_description {
    include!(concat!(env!("OUT_DIR"), "/graph_description.rs"));
}


custom_derive! {
    #[derive(NewtypeFrom, NewtypeDeref, NewtypeDerefMut, Debug, Clone)]
    pub struct ProcessDescription(ProcessDescriptionProto);
}

custom_derive! {
    #[derive(NewtypeFrom, NewtypeDeref, NewtypeDerefMut, Debug, Clone)]
    pub struct FileDescription(FileDescriptionProto);
}

custom_derive! {
    #[derive(NewtypeFrom, NewtypeDeref, NewtypeDerefMut, Debug, Clone)]
    pub struct IpAddressDescription(IpAddressDescriptionProto);
}

custom_derive! {
    #[derive(NewtypeFrom, NewtypeDeref, NewtypeDerefMut, Debug, Clone)]
    pub struct EdgeDescription(EdgeDescriptionProto);
}

custom_derive! {
    #[derive(NewtypeFrom, NewtypeDeref, NewtypeDerefMut, Clone, Debug)]
    pub struct NodeDescription(NodeDescriptionProto);
}

custom_derive! {
    #[derive(NewtypeFrom, NewtypeDeref, NewtypeDerefMut, Clone, Debug)]
    pub struct GraphDescription(GraphDescriptionProto);
}

custom_derive! {
    #[derive(NewtypeFrom, NewtypeDeref, NewtypeDerefMut, Clone, Debug)]
    pub struct OutboundConnection(OutboundConnectionProto);
}

custom_derive! {
    #[derive(NewtypeFrom, NewtypeDeref, NewtypeDerefMut, Clone, Debug)]
    pub struct InboundConnection(InboundConnectionProto);
}

custom_derive! {
    #[derive(NewtypeFrom, NewtypeDeref, NewtypeDerefMut, Clone, Debug)]
    pub struct GeneratedSubgraphs(GeneratedSubgraphsProto);
}

impl GeneratedSubgraphs {
    pub fn new(subgraphs: Vec<GraphDescription>) -> GeneratedSubgraphs {
        GeneratedSubgraphsProto {
            subgraphs: subgraphs.into_iter().map(GraphDescription::into).collect()
        }.into()
    }
}

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
        host_id: HostIdentifier,
        state: ConnectionState,
        port: u32,
        timestamp: u64,
    ) -> OutboundConnection {
        OutboundConnectionProto {
            node_key: Uuid::new_v4().to_string(),
            host_id: Some(host_id.into()),
            state: state.into(),
            port,
            timestamp,
        }.into()
    }

    pub fn clone_key(&self) -> String {
        self.node_key.clone()
    }

    pub fn into_json(self) -> Value {
        let asset_id = &self.host_id.as_ref().unwrap().host_id.as_ref().unwrap();
        let asset_id = match asset_id {
            HostId::AssetId(asset_id) => asset_id,
            _ => unimplemented!()
        };

        let mut j = json!({
            "node_key": self.node_key,
            "asset_id": asset_id,
            "port": self.port,
            "timestamp": self.timestamp,
            "direction": "outbound",
        });


        match ConnectionState::from(self.state) {
            ConnectionState::Created => j["create_time"] = self.timestamp.into(),
            ConnectionState::Terminated => j["terminate_time"] = self.timestamp.into(),
            ConnectionState::Existing => j["seen_at"] = self.timestamp.into(),
        };
        j
    }

    pub fn set_key(&mut self, key: String) {
        self.node_key = key;
    }

    pub fn set_asset_id(&mut self, asset_id: String) {
        self.host_id = Some(
            Host{
                host_id: Some(HostId::AssetId(asset_id))
            }
        )
    }

    pub fn asset_id(&self) -> &str {
        let asset_id = &self.host_id.as_ref().unwrap().host_id.as_ref().unwrap();
        match asset_id {
            HostId::AssetId(asset_id) => asset_id,
            _ => panic!("Must attribute before calling asset_id")
        }
    }
}


impl InboundConnection {
    pub fn new(
        host_id: HostIdentifier,
        state: ConnectionState,
        port: u32,
        timestamp: u64,
    ) -> InboundConnection {
        InboundConnectionProto {
            node_key: Uuid::new_v4().to_string(),
            host_id: Some(host_id.into()),
            state: state.into(),
            port,
            timestamp,
        }.into()
    }

    pub fn clone_key(&self) -> String {
        self.node_key.clone()
    }

    pub fn into_json(self) -> Value {
        let asset_id = &self.host_id.as_ref().unwrap().host_id.as_ref().unwrap();
        let asset_id = match asset_id {
            HostId::AssetId(asset_id) => asset_id,
            _ => unimplemented!()
        };
        let mut j = json!({
            "node_key": self.node_key,
            "asset_id": asset_id,
            "port": self.port,
            "timestamp": self.timestamp,
            "direction": "inbound",
        });


        match ConnectionState::from(self.state) {
            ConnectionState::Created => j["create_time"] = self.timestamp.into(),
            ConnectionState::Terminated => j["terminate_time"] = self.timestamp.into(),
            ConnectionState::Existing => j["seen_at"] = self.timestamp.into(),
        };
        j
    }

    pub fn set_key(&mut self, key: String) {
        self.node_key = key;
    }

    pub fn set_asset_id(&mut self, asset_id: String) {
        self.host_id = Some(
            Host{
                host_id: Some(HostId::AssetId(asset_id))
            }
        )
    }

    pub fn asset_id(&self) -> &str {
        let asset_id = &self.host_id.as_ref().unwrap().host_id.as_ref().unwrap();
        match asset_id {
            HostId::AssetId(asset_id) => asset_id,
            _ => panic!("Must attribute before calling asset_id")
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
        impl From<$t> for NodeDescriptionProto {
            fn from(t: $t) -> Self {
                NodeDescriptionProto {
                    which_node: WhichNode::$n(
                        t.into()
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
}


impl NodeDescriptionProto {

    pub fn get_timestamp(&self) -> u64 {
        match self.which_node.as_ref().unwrap() {
            WhichNode::ProcessNode(ref node) => {
                node.timestamp
            }
            WhichNode::FileNode(ref node) => {
                node.timestamp
            }
            WhichNode::IpAddressNode(ref node) => {
                node.timestamp
            }
            WhichNode::OutboundConnectionNode(ref node) => {
                node.timestamp
            }
            WhichNode::InboundConnectionNode(ref node) => {
                node.timestamp
            }
        }
    }

    pub fn get_key(&self) -> &str {
        match self.which_node.as_ref().unwrap() {
            WhichNode::ProcessNode(ref node) => {
                &node.node_key
            }
            WhichNode::FileNode(ref node) => {
                &node.node_key
            }
            WhichNode::IpAddressNode(ref node) => {
                &node.node_key
            }
            WhichNode::OutboundConnectionNode(ref node) => {
                &node.node_key
            }
            WhichNode::InboundConnectionNode(ref node) => {
                &node.node_key
            }
        }
    }

    pub fn set_asset_id(&mut self, asset_id: String) {
        match self.which_node.as_mut().unwrap() {
            WhichNode::ProcessNode(ref mut node) => {
                node.host_id = Some(
                    Host{
                        host_id: Some(HostId::AssetId(asset_id))
                    }
                )
            }
            WhichNode::FileNode(ref mut node) => {
                node.host_id = Some(
                    Host{
                        host_id: Some(HostId::AssetId(asset_id))
                    }
                )
            }
            WhichNode::IpAddressNode(ref mut node) => {
                panic!("ip address node has no asset id")
            }
            WhichNode::OutboundConnectionNode(ref mut node) => {
                node.host_id = Some(
                    Host{
                        host_id: Some(HostId::AssetId(asset_id))
                    }
                )
            }
            WhichNode::InboundConnectionNode(ref mut node) => {
                node.host_id = Some(
                    Host{
                        host_id: Some(HostId::AssetId(asset_id))
                    }
                )
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
}

#[derive(Debug, Clone)]
pub enum ProcessState {
    Created,
    Terminated,
    Existing
}

impl Into<u32> for ProcessState {
    fn into(self) -> u32 {
        match self {
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


impl Into<u32> for FileState {
    fn into(self) -> u32 {
        match self {
            FileState::Created => 1,
            FileState::Deleted => 2,
            FileState::Existing => 3,
        }
    }
}

impl ProcessDescription {
    pub fn new(host_id: HostIdentifier,
               state: ProcessState,
               pid: u64,
               timestamp: u64,
               image_name: Vec<u8>,
               image_path: Vec<u8>
    ) -> ProcessDescription {
        ProcessDescriptionProto {
            node_key: Uuid::new_v4().to_string(),
            host_id: Some(host_id.into()),
            state: state.into(),
            pid,
            timestamp,
            image_name,
            image_path,
        }.into()
    }

    pub fn get_key(&self) -> &str {
        &self.node_key
    }

    pub fn set_key(&mut self, key: String) {
        self.node_key = key;
    }

    pub fn set_asset_id(&mut self, asset_id: String) {
        self.host_id = Some(
            Host{
                host_id: Some(HostId::AssetId(asset_id))
            }
        )
    }

    pub fn clone_key(&self) -> String {
        self.node_key.clone()
    }

    pub fn into_json(self) -> Value {
        let asset_id = &self.host_id.as_ref().unwrap().host_id.as_ref().unwrap();
        let asset_id = match asset_id {
            HostId::AssetId(asset_id) => asset_id,
            _ => unimplemented!()
        };
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
            ProcessState::Created => j["create_time"] = self.timestamp.into(),
            ProcessState::Terminated => j["terminate_time"] = self.timestamp.into(),
            ProcessState::Existing => j["seen_at"] = self.timestamp.into(),
        }

        j
    }

    pub fn asset_id(&self) -> &str {
        let asset_id = &self.host_id.as_ref().unwrap().host_id.as_ref().unwrap();
        match asset_id {
            HostId::AssetId(asset_id) => asset_id,
            _ => unimplemented!()
        }
    }
}

impl OutboundConnectionProto {
    pub fn asset_id(&self) -> &str {
        let asset_id = &self.host_id.as_ref().unwrap().host_id.as_ref().unwrap();
        match asset_id {
            HostId::AssetId(asset_id) => asset_id,
            _ => unimplemented!()
        }
    }
}

impl InboundConnectionProto {
    pub fn asset_id(&self) -> &str {
        let asset_id = &self.host_id.as_ref().unwrap().host_id.as_ref().unwrap();
        match asset_id {
            HostId::AssetId(asset_id) => asset_id,
            _ => unimplemented!()
        }
    }
}

impl ProcessDescriptionProto {
    pub fn asset_id(&self) -> &str {
        let asset_id = &self.host_id.as_ref().unwrap().host_id.as_ref().unwrap();
        match asset_id {
            HostId::AssetId(asset_id) => asset_id,
            _ => unimplemented!()
        }
    }
}

impl FileDescriptionProto {
    pub fn asset_id(&self) -> &str {
        let asset_id = &self.host_id.as_ref().unwrap().host_id.as_ref().unwrap();
        match asset_id {
            HostId::AssetId(asset_id) => asset_id,
            _ => unimplemented!()
        }
    }
}

impl FileDescription {
    pub fn new(host_id: HostIdentifier,
               state: FileState,
               timestamp: u64,
               path: Vec<u8>,
    ) -> FileDescription {
        FileDescriptionProto {
            node_key: Uuid::new_v4().to_string(),
            host_id: Some(host_id.into()),
            state: state.into(),
            timestamp,
            path
        }.into()
    }

    pub fn get_key(&self) -> &str {
        &self.node_key
    }

    pub fn set_key(&mut self, key: String) {
        self.node_key = key;
    }

    pub fn set_asset_id(&mut self, asset_id: String) {
        self.host_id = Some(
            Host{
                host_id: Some(HostId::AssetId(asset_id))
            }
        )
    }

    pub fn clone_key(&self) -> String {
        self.node_key.clone()
    }

    pub fn into_json(self) -> Value {
        let asset_id = &self.host_id
            .as_ref().unwrap().host_id.as_ref().unwrap();
        let asset_id = match asset_id {
            HostId::AssetId(asset_id) => asset_id,
            _ => unimplemented!()
        };
        let mut j = json!({
            "node_key": self.node_key,
            "asset_id": asset_id,
        });

        if !self.path.is_empty() {
            j["path"] = Value::from(String::from_utf8_lossy(&self.path));
        }
        match FileState::from(self.state) {
            FileState::Created => j["create_time"] = self.timestamp.into(),
            FileState::Deleted => j["terminate_time"] = self.timestamp.into(),
            FileState::Existing => j["seen_at"] = self.timestamp.into(),
        }

        j
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

        IpAddressDescriptionProto {
            node_key,
            timestamp,
            ip_address
        }.into()
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

}

impl GraphDescription {
    pub fn new(timestamp: u64) -> Self {
        GraphDescriptionProto {
            nodes: hashmap![],
            edges: hashmap![],
            timestamp
        }.into()
    }


    pub fn add_node<N>(&mut self, node: N)
        where N: Into<NodeDescriptionProto>
    {
        let node = node.into();
        let key = node.get_key().to_owned();

        self.nodes.insert(key.clone(), node);
        self.edges
            .entry(key)
            .or_insert_with(|| {
                EdgeListProto { edges: vec![] }.into()
            });
    }


    pub fn with_node<N>(self, node: N) -> GraphDescription
        where N: Into<NodeDescriptionProto>
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
        let edge = EdgeDescriptionProto {
            from_neighbor_key: from.clone(),
            to_neighbor_key: to,
            edge_name
        }.into();

        self.edges
            .entry(from)
            .or_insert_with(|| {
                EdgeListProto { edges: Vec::with_capacity(1) }.into()
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


