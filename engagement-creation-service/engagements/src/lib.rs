#![feature(nll)]
#[macro_use] extern crate serde_derive;

#[macro_use]
extern crate serde_json;
extern crate serde;

extern crate dgraph_client;
extern crate incident_graph;

extern crate uuid;

use uuid::Uuid;

use incident_graph::*;

use dgraph_client::api_grpc::DgraphClient;

use std::collections::HashMap;
use std::collections::HashSet;


pub enum ENode {
    File(EFile),
    Process(EProcess),
}

impl From<EFile> for ENode {
    fn from(f: EFile) -> ENode {
        ENode::File(f)
    }
}

impl From<EProcess> for ENode {
    fn from(p: EProcess) -> ENode {
        ENode::Process(p)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EFile {
    pub node_key: String,
    pub asset_id: String,
    pub create_time: u64,
    pub delete_time: Option<u64>,
    pub path: String,
    pub engagement_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EProcess {
    pub node_key: String,
    pub pid: u64,
    pub create_time: u64,
    pub asset_id: String,
    pub terminate_time: Option<String>,
    pub image_name: Option<String>,
    pub engagement_id: String,
}

#[derive(Hash, PartialEq, Eq)]
pub struct Edge {
    pub from: String,
    pub to: String,
    pub edge_name: String,
}

pub struct EGraph {
    pub nodes: HashMap<String, ENode>,
    pub edges: HashMap<String, HashSet<Edge>>,
    pub engagement_id: String,
}

impl EGraph {

    pub fn new() -> EGraph {
        EGraph {
            nodes: HashMap::new(),
            edges: HashMap::new(),
            engagement_id: Uuid::new_v4().to_string()
        }
    }

    pub fn add_file(&mut self, file: File) {
        let node_key = file.node_key.clone();
        let efile = EFile {
            node_key: node_key.clone(),
            asset_id: file.asset_id,
            create_time: file.create_time,
            delete_time: file.delete_time,
            path: file.path,
            engagement_id: self.engagement_id.clone(),
        };

        self.nodes.insert(node_key.clone(), efile.into());

        if let Some(creator) = file.creator {

            let edge = Edge {
                from: node_key.clone(),
                to: creator.node_key.clone(),
                edge_name: "creator".into()
            };

            let edge_exists = self.edges.get(&node_key).map(|edges| {
                edges.contains(&edge)
            }).unwrap_or(false);


            if !edge_exists {
                self.edges.entry(node_key.clone())
                    .or_insert_with(|| HashSet::new())
                    .insert(
                        edge
                    );
                self.add_process(*creator);
            }
        }
    }

    pub fn add_process(&mut self, process: Process) {
        let node_key = process.node_key.clone();
        let eprocess = EProcess {
            node_key: node_key.clone(),
            pid: process.pid,
            create_time: process.create_time,
            asset_id: process.asset_id,
            terminate_time: process.terminate_time,
            image_name: process.image_name,
            engagement_id: self.engagement_id.clone(),
        };

        self.nodes.insert(node_key.clone(), eprocess.into());

        for child in process.children.into_iter() {

            let edge = Edge {
                from: node_key.clone(),
                to: child.node_key.clone(),
                edge_name: "children".into()
            };

            let edge_exists = self.edges.get(&node_key).map(|edges| {
                edges.contains(&edge)
            }).unwrap_or(false);

            if !edge_exists {
                self.edges.entry(node_key.clone())
                    .or_insert_with(|| HashSet::new())
                    .insert(
                        edge
                    );
                self.add_process(child);
            }
        }

        if let Some(bin_file) = process.bin_file {
            let bin_file_key = bin_file.node_key.clone();

            let edge = Edge {
                from: node_key.clone(),
                to: bin_file.node_key.clone(),
                edge_name: "bin_file".into()
            };

            let edge_exists = self.edges.get(&node_key).map(|edges| {
                edges.contains(&edge)
            }).unwrap_or(false);

            if !edge_exists {
                self.edges.entry(node_key.clone())
                    .or_insert_with(|| HashSet::new())
                    .insert(
                        edge
                    );
                self.add_file(*bin_file);
            }

        }
    }
}
