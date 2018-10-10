#![feature(nll)]
#[macro_use] extern crate serde_derive;


#[macro_use]
extern crate log;

#[macro_use]
extern crate serde_json;
extern crate serde;

extern crate dgraph_client;
extern crate failure;

extern crate uuid;

extern crate sha2;
use serde_json::Value;
use uuid::Uuid;
use failure::Error;

use dgraph_client::api_grpc::DgraphClient;
use sha2::{Digest, Sha256};

use std::collections::HashMap;
use std::collections::HashSet;

const PROCESS_ATTRIBUTES: &[&str] = &[
    "uid",
    "node_key",
    "pid",
    "create_time",
    "asset_id",
    "terminate_time",
    "image_name",
    "arguments",
];

const PROCESS_ATTRIBUTES_COMMA_SEP: &str =
    "uid, node_key, pid, create_time, asset_id, terminate_time,
    image_name, arguments";



const FILE_ATTRIBUTES: &[&str] = &[
    "uid",
    "node_key",
    "asset_id",
    "create_time",
    "delete_time",
    "path",
];

const FILE_ATTRIBUTES_COMMA_SEP: &str =
    "uid,
    node_key,
    asset_id,
    create_time,
    delete_time,
    path";


#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RootNode {
    File(File),
    Process(Process)
}

impl RootNode {
    pub fn get_node_key(&self) -> &str {
        match self {
            RootNode::File(ref file) => &file.node_key,
            RootNode::Process(ref process) => &process.node_key,
        }
    }
}

pub fn root_node_hash(root_node: &RootNode) -> Vec<u8> {
    let mut hasher = Sha256::default();
    let mut uids: Vec<&str> = vec![];

    match root_node {
        RootNode::File(ref file) => {
            let uid = &file.uid;
            uids.push(uid);
        },
        RootNode::Process(ref process) => {
            get_proc_uids(&process, &mut uids);
        }
    };

    uids.sort_unstable();

    for uid in uids {
        hasher.input(uid);
    }

    hasher.result().to_vec()
}

fn get_proc_uids<'a>(process: &'a Process, uids: &mut Vec<&'a str>) {
    uids.push(&process.uid);
    for child in process.children.iter() {
        get_proc_uids(&child, uids);
    }
    if let Some(ref bin_file) = process.bin_file {
        get_file_uids(&bin_file, uids)
    }
}

fn get_file_uids<'a>(file: &'a File, uids: &mut Vec<&'a str>) {
    uids.push(&file.uid);
}

impl From<Process> for RootNode {
    fn from(process: Process) -> RootNode {
        RootNode::Process(process)
    }
}

impl From<File> for RootNode {
    fn from(file: File) -> RootNode {
        RootNode::File(file)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct File {
    #[serde(skip_serializing)]
    pub uid: String,
    pub node_key: String,
    pub asset_id: String,
    pub create_time: u64,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delete_time: Option<u64>,
    pub path: String,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub creator: Option<Box<Process>>
}

impl File {
    pub fn add_creator(&mut self, dgraph_client: &DgraphClient) {
        info!("Adding creator");
        let mut req = dgraph_client::api::Request::new();

        let query = format!(
            r#"{{
                    q(func: uid("{}")) {{
                        uid,
                        ~created_files {{
                          {}
                        }}
                    }}
                }}"#, self.uid, PROCESS_ATTRIBUTES_COMMA_SEP).to_owned();

        req.query = query;

        info!("Query for adding creator: {}", &req.query);

        let resp = dgraph_client.query(&req).expect("query");

        let resp: serde_json::Value =
            serde_json::from_slice(resp.get_json()).unwrap();
        info!("{}", resp);

        let resp = &resp["q"][0];
        if resp.is_null() {
            return
        }

        let encoded_p = &resp["q"][0]["~created_files"][0];

        if encoded_p.is_null() {
            return
        }

        let p: Process =
            serde_json::from_value(encoded_p.clone()).unwrap();

        self.creator = Some(Box::new(p));
    }


    pub fn procs_executed_from(f: &File,
                               dgraph_client: &DgraphClient) -> Vec<Process> {
        let mut req = dgraph_client::api::Request::new();

        let query = format!(
            r#"{{
                q(func: uid({})) {{
                    ~bin_file {{
                        {}
                    }}
                }}
            }}"#, &f.uid, PROCESS_ATTRIBUTES_COMMA_SEP).to_owned();


        let resp = dgraph_client.query(&req).expect("query");

        let resp: serde_json::Value =
            serde_json::from_slice(resp.get_json())
                .expect("Could not convert slice to Value for procs_executed_from");

        serde_json::from_value(resp["q"].clone()).unwrap()
    }

}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Process {
    #[serde(skip_serializing)]
    pub uid: String,
    #[serde(default)]
    pub children: Vec<Process>,
    pub node_key: String,
    pub pid: u64,
    pub create_time: u64,
    pub asset_id: String,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub terminate_time: Option<String>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_name: Option<String>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bin_file: Option<Box<File>>,
}

impl Process {
    pub fn map_children(&mut self, f: impl Fn(&mut Self) + Clone) {
        f(self);
        for child in self.children.iter_mut() {
            Process::map_children(child, f.clone());
        }
    }

    pub fn add_children(&mut self, dgraph_client: &DgraphClient) {
        let mut req = dgraph_client::api::Request::new();

        let query = format!(
            r#"{{
                q(func: uid({})) {{
                    children {{
                    {}
                    }}
                }}
            }}"#, self.uid, PROCESS_ATTRIBUTES_COMMA_SEP).to_owned();

        req.query = query;

        let resp = dgraph_client.query(&req).expect("query");

        let resp: Value =
            serde_json::from_slice(resp.get_json()).expect("invalid json children");

        let resp = &resp["q"][0];

        info!("add_children response is {}", resp);

        let children_json = &resp["children"];

        if children_json.is_null() {
            return
        }

        let children =
            serde_json::from_value(children_json.clone()).expect("Failed to parse children as Vec<Process>");

        self.children = children;
    }

    pub fn add_file(&mut self, dgraph_client: &DgraphClient) {
        let mut req = dgraph_client::api::Request::new();

        let query = format!(
            r#"{{
                q(func: uid({})) {{
                    bin_file {{
                        {}
                    }}
                }}
            }}"#, self.uid, FILE_ATTRIBUTES_COMMA_SEP).to_owned();

        info!("Adding file with: {}", query);
        req.query = query;

        let resp = dgraph_client.query(&req).expect("query");

        let resp: serde_json::Value =
            serde_json::from_slice(resp.get_json()).unwrap();

        info!("{}", &resp["q"]);
        if resp["q"][0].is_null() {
            return;
        }

        let f: File = serde_json::from_value(resp["q"][0]["bin_file"][0].clone())
            .expect("Failed to deserialize File when adding file to process");

        self.bin_file = Some(Box::new(f));
    }


}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ERootNode {
    EFile(EFile),
    EProcess(EProcess)
}

impl From<EProcess> for ERootNode {
    fn from(process: EProcess) -> ERootNode {
        ERootNode::EProcess(process)
    }
}

impl From<EFile> for ERootNode {
    fn from(file: EFile) -> ERootNode {
        ERootNode::EFile(file)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EFile {
    pub uid: String,
    pub node_key: String,
    pub engagement_key: String,
    pub asset_id: String,
    pub create_time: u64,
    pub delete_time: Option<u64>,
    pub path: String,
    pub creator: Option<Box<EProcess>>,
}

impl EFile {
    pub fn add_creator(&mut self, dgraph_client: &DgraphClient) {
        let mut req = dgraph_client::api::Request::new();

        let query = format!(
            r#"{{
                q(func: has(pid)) {{
                    {},
                    ~created
                    @filter(uid({})))
                }}
            }}"#, PROCESS_ATTRIBUTES_COMMA_SEP, self.uid).to_owned();

        req.query = query;

        let resp = dgraph_client.query(&req).expect("query");

        let resp: serde_json::Value =
            serde_json::from_slice(resp.get_json()).unwrap();

        let p: EProcess =
            serde_json::from_value(resp["q"][0].clone()).unwrap();

        self.creator = Some(Box::new(p));
    }

}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EProcess {
    pub uid: String,
    pub node_key: String,
    pub engagement_key: String,
    #[serde(default)]
    pub children: Vec<EProcess>,
    pub pid: u64,
    pub create_time: u64,
    pub asset_id: String,
    pub terminate_time: Option<String>,
    pub image_name: Option<String>,
    pub bin_file: Option<Box<EFile>>,
}

impl EProcess {
    pub fn from_process(
        dgraph_client: &DgraphClient,
        process: &Process,
        engagement_key: impl AsRef<str>
    ) -> EProcess {

        let process_bytes = json!({
            "engagement_key": engagement_key.as_ref(),
            "node_key": &process.node_key,
            "pid": process.pid,
            "create_time": process.create_time,
            "asset_id": process.asset_id,
            "terminate_time": process.terminate_time,
            "image_name": process.image_name,
        }).to_string().into_bytes();

        upsert_node(
            dgraph_client,
            &process.node_key,
            engagement_key,
            process_bytes
        );
        unimplemented!()
    }
}


pub fn get_children(dgraph_client: &DgraphClient,
                    node_key: impl AsRef<str>) -> Vec<Process> {
    let mut req = dgraph_client::api::Request::new();

    let query = format!(
        r#"{{
                q(func: eq(node_key, "{}")) {{
                    children {{
                        {}
                    }}
                }}
            }}"#, node_key.as_ref(), PROCESS_ATTRIBUTES_COMMA_SEP);

    req.query = query;

    let resp = dgraph_client.query(&req).expect("query");

    serde_json::from_slice(resp.get_json()).unwrap()

}

pub fn get_parent(dgraph_client: &DgraphClient,
                  node_key: impl AsRef<str>) -> Process {
    let mut req = dgraph_client::api::Request::new();

    let query = format!(
        r#"{{
                q(func: has(pid)) {{
                    children
                    @filter(eq(node_key, "{}"))
                    {{
                        {}
                    }}
                }}
            }}"#, node_key.as_ref(), PROCESS_ATTRIBUTES_COMMA_SEP);

    req.query = query;

    let resp = dgraph_client.query(&req).expect("query");

    serde_json::from_slice(resp.get_json()).unwrap()

}

// Given the expanded graph we've collected from the master graph, store the
// data in the Engagements Graph
pub fn insert_root_node(dgraph_client: &DgraphClient,
                        engagement_key: impl AsRef<str>,
                        root_node: impl Into<RootNode>
) -> Result<(), Error> {

    let engagement_key = engagement_key.as_ref();
    let root_node = root_node.into();

    match root_node {
        RootNode::Process(ref process) => {
            let mut edges = Vec::with_capacity(100);

            insert_process(
                dgraph_client,
                process,
                engagement_key,
                &mut edges,
            );
        }
        RootNode::File(ref file) => {
            let mut edges = Vec::with_capacity(100);

            insert_file(
                dgraph_client,
                file,
                engagement_key,
                &mut edges,
            );
        }
    }

    Ok(())
}

fn insert_edges(
    dgraph_client: &DgraphClient,
    edges: &[(String, &'static str, String)]
) {
    // TODO: Generate inserts, then do it in one request
    for (from, edge_name, to) in edges {
        upsert_edge(
            dgraph_client,
            from,
            to,
            edge_name
        )
    }
}

fn insert_process(dgraph_client: &DgraphClient,
                  process: &Process,
                  engagement_key: &str,
                  edges: &mut Vec<(String, &'static str, String)>) -> String {

    let process_bytes = json!({
        "engagement_key": engagement_key,
        "node_key": process.node_key,
        "pid": process.pid,
        "create_time": process.create_time,
        "asset_id": process.asset_id,
        "terminate_time": process.terminate_time,
        "image_name": process.image_name,
    }).to_string().into_bytes();

    let process_uid = upsert_node(
        dgraph_client,
        &process.node_key,
        engagement_key,
        process_bytes
    );

    // Insert child nodes
    for child in &process.children {
        let child_uid = insert_process(
            dgraph_client,
            process,
            engagement_key,
            edges
        );
        edges.push((process_uid.to_owned(), "children", child_uid.clone()));
    }

    process_uid
}

fn insert_file(dgraph_client: &DgraphClient,
               file: &File,
               engagement_key: &str,
               edges: &mut Vec<(String, &'static str, String)>)
               -> String {

    let file_bytes = json!({
        "engagement_key": engagement_key,
        "node_key": file.node_key,
        "asset_id": file.asset_id,
        "create_time": file.create_time,
        "delete_time": file.delete_time,
        "path": file.path,
    }).to_string().into_bytes();

    let file_uid = upsert_node(dgraph_client,
                               &file.node_key,
                               engagement_key,
                               file_bytes);

    if let Some(ref creator) = file.creator {
        let creator_uid = insert_process(
            dgraph_client,
            creator,
            engagement_key,
            edges
        );

        edges.push((file_uid.clone(), "creator", creator_uid));
    }

    file_uid
}

fn upsert_edge(dgraph_client: &DgraphClient,
               to: impl AsRef<str>,
               from: impl AsRef<str>,
               edge_name: impl AsRef<str>) {
    let to = to.as_ref();
    let from = from.as_ref();
    let edge_name = edge_name.as_ref();

    let mut req = dgraph_client::api::Request::new();

    // Get the uid for the node with `node_key`
    req.query = format!(r#"
                {{
                    question(func: uid({}))
                    {{
                        {} @filter(uid({})) {{
                            uid
                        }}
                    }}
                }}"#, from, edge_name, to);

    let resp = dgraph_client.query(&req).expect("query");

    let uid: serde_json::Value =
        serde_json::from_slice(resp.get_json()).unwrap();

    let uid = uid["question"][0]
        .get("children")
        .and_then(|children| children.as_object())
        .and_then(|children| children.get("uid"))
        .and_then(|uid| uid.as_str())
        .clone();


    info!("uid is {:#?}", uid);
    // If the child doesn't exist, create the edge
    if uid.is_none() {
        let mut mutation = dgraph_client::api::Mutation::new();
        let m = json!(
                        {
                            "uid": from,
                            edge_name: {
                                "uid": to
                            }
                        }
                    ).to_string();

        mutation.commit_now = true;
        mutation.set_json = m.into_bytes();

        loop {
            let mut_res = dgraph_client.mutate(&mutation);
            match mut_res {
                Ok(_) => break,
                Err(e) => error!("{}", e)
            }
        }
    }

}

fn upsert_node(dgraph_client: &DgraphClient,
               node_key: impl AsRef<str>,
               engagement_key: impl AsRef<str>,
               node: Vec<u8>) -> String {

    let node_key = node_key.as_ref();
    let engagement_key = engagement_key.as_ref();

    let mut req = dgraph_client::api::Request::new();

    // Get the uid for the node with `node_key`
    req.query = format!(r#"
            {{
                question(func: eq(node_key, "{}"))
                @filter(eq(engagement_key, "{}"))
                {{
                    uid,
                }}
            }}"#, node_key, engagement_key);


    let resp = dgraph_client.query(&req).expect("query");
    let uid: serde_json::Value =
        serde_json::from_slice(resp.get_json()).unwrap();

    let uid = uid["question"][0]
        .get("uid")
        .and_then(|uid| uid.as_str()).clone();


    // If the `node_key` maps to an existing node, use the node's `_uid_`
    // Otherwise (None), add the node, and get back the new `_uid_` for it
    info!("upsert node uid: {:#?}", uid);
    match uid {
        Some(uid) => {
            uid.to_string()
        }
        None => {
            // Add node
            let mut mutation = dgraph_client::api::Mutation::new();
            mutation.commit_now = true;
            mutation.set_json = node;

            loop {
                let mut_res = dgraph_client.mutate(&mutation);
                match mut_res {
                    Ok(mut_res) => {
                        let uid =
                            mut_res
                                .get_uids()
                                .get("blank-0")
                                .unwrap();
                        break uid.to_owned();
                    }
                    Err(e) => {
                        error!("mutation error {:#?}", e);
                        continue;
                    }
                }
            }
        }
    }
}


pub fn set_engagement_process_schema(client: &DgraphClient) {
    let mut op_schema = dgraph_client::api::Operation::new();
    op_schema.schema = r#"
       		node_key: string @upsert @index(hash) .
       		engagement_key: string @index(hash) .
       		pid: int @index(int) .
       		create_time: int @index(int) .
       		asset_id: string @index(hash) .
       		terminate_time: int @index(int) .
       		image_name: string @index(hash) .
       		arguments: string .
       		analyzer_names: [string] .

       		bin_file: uid @reverse .
       		children: uid @reverse .
       		created_files: uid @reverse .
            deleted_files: uid @reverse .
            read_files: uid @reverse .
            wrote_files: uid @reverse .
        "#.to_string();
    let res = client.alter(&op_schema).expect("set schema");
}

pub fn set_engagement_file_schema(client: &DgraphClient) {
    let mut op_schema = dgraph_client::api::Operation::new();
    op_schema.schema = r#"
       		node_key: string @upsert @index(hash) .
       		engagement_key: string @index(hash) .
       		asset_id: string @index(hash) .
       		create_time: int @index(int) .
       		delete_time: int @index(int) .
       		path: string @index(hash) .
        "#.to_string();
    let res = client.alter(&op_schema).expect("set schema");
}
