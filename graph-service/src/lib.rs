extern crate base64;
extern crate dgraph_client;
extern crate failure;
extern crate futures_await as futures;
extern crate graph_descriptions;
extern crate hash_hasher;
#[macro_use]
extern crate log;
extern crate prost;
#[macro_use]
extern crate prost_derive;
extern crate rusoto_core;
extern crate rusoto_s3;
#[macro_use]
extern crate serde_json;
extern crate sha2;
extern crate stopwatch;

use dgraph_client::api_grpc::DgraphClient;
use failure::Error;
use futures::future::Future;
use graph_descriptions::*;
use graph_descriptions::graph_description::*;
use hash_hasher::{HashBuildHasher, HashMap, HashSet};
use prost::Message;
use rusoto_core::Region;
use rusoto_s3::{PutObjectRequest, S3, S3Client};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::cmp::max;
use std::cmp::min;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use stopwatch::Stopwatch;


macro_rules! log_time {
    ($msg: expr, $x:expr) => {
        let mut sw = Stopwatch::start_new();
        #[allow(path_statements)]
        let result = {$x};
        sw.stop();
        info!("{} {} milliseconds", $msg, sw.elapsed_ms());
        result
    };
}

pub mod subgraph_merge_event {
    include!(concat!(env!("OUT_DIR"), "/subgraph_merge_event.rs"));
}

pub fn upsert_with_retries(client: &DgraphClient, node: &NodeDescriptionProto) -> Result<String, Error> {
    let mut retries = 5;
    loop {
        let res = upsert_node(client, node);
        match res {
            ok @ Ok(_) => break ok,
            Err(e) => {
                if retries == 0 {
                    break Err(e)
                }
                warn!("Upsert failed, retrying: {}", e);
                retries -= 1;
            }
        }
    }
}


pub fn merge_subgraph(client: &DgraphClient, subgraph: &GraphDescription)
                      -> Result<(u64, u64), Error> {

    let mut node_key_uid_map = HashMap::with_capacity_and_hasher(subgraph.nodes.len(),
                                                                 HashBuildHasher::default());

    let mut earliest = u64::max_value();
    let mut latest = 0u64;

    log_time!{
        "upsert_nodes",
        for node in subgraph.nodes.iter() {
            earliest = min(earliest, node.1.get_timestamp());
            latest = max(latest, node.1.get_timestamp());

            let (node_key, node) = node;
            info!("Upserting node");
            let node_uid = upsert_with_retries(&client, node)?;
            info!("Upserted node");

            node_key_uid_map.insert(node_key.as_ref(), node_uid);
        }
    };


    info!("earliest: {} latest: {}", earliest, latest);

    for edges in subgraph.edges.iter() {
        for edge in edges.1.edges.iter() {
            let to = &node_key_uid_map
                .get(edge.to_neighbor_key.as_str())
                .expect("edge.to_neighbor_key");
            let from = &node_key_uid_map
                .get(edge.from_neighbor_key.as_str())
                .expect("edge.from_neighbor_key");
            let edge_name = edge.edge_name.clone();

            let mut req = dgraph_client::api::Request::new();

            // Get the uid for the node with `node_key`
            req.query = format!(r#"
                {{
                    question(func: eq(node_key, "{}"))
                    {{
                        children @filter(uid({})) {{
                            uid
                        }}
                    }}
                }}"#, from, to);


//            let resp = log_time!(
//                    "dgraph_node_key_query",
//                    client.query(&req).expect("query")
//                );

            let resp = client.query(&req).expect("query");

            let uid: serde_json::Value = serde_json::from_slice(resp.get_json()).unwrap();
            let uid = uid["question"][0]
                .get("children")
                .and_then(|children| children.as_object())
                .and_then(|children| children.get("uid"))
                .and_then(|uid| uid.as_str())
                .clone();


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

                info!("edge: {}", m);

                mutation.commit_now = true;
                mutation.set_json = m.into_bytes();

//                log_time!{
//                    "mutation",
                    loop {
                        let mut_res = client.mutate(&mutation);
                        match mut_res {
                            Ok(_) => break,
                            Err(e) => error!("{}", e)
                        }
                    }
//                }

            }
        }
    }

    Ok((earliest, latest))
}

pub fn upsert_node(client: &DgraphClient, node: &NodeDescriptionProto) -> Result<String, Error> {
    let node_key = node.get_key();
    let mut txn = dgraph_client::api::TxnContext::new();

    txn.set_start_ts(SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?.as_secs());
    // Get the _uid_ associated with our generated node_key
    let mut req = dgraph_client::api::Request::new();

    // TODO: Check what type the 'node' is and query for all fields
    // Get the uid for the node with `node_key`
    req.query = format!(r#"
            {{
                question(func: eq(node_key, "{}"))
                {{
                    uid,
                }}
            }}"#, node_key);

    let resp = client.query(&req).expect("upsert query");
    let json_response: serde_json::Value = serde_json::from_slice(resp.get_json()).unwrap();

    info!("response: {}", json_response);
    let uid = json_response["question"][0]
        .get("uid")
        .and_then(|uid| uid.as_str()).clone();
    info!("got uid {:#?}", uid);


    let new_node_key: String = match uid {
        Some(uid) => {
            let mut mutation = dgraph_client::api::Mutation::new();
            mutation.commit_now = true;
            let mut json_node = (*node).clone().into_json();
            json_node["uid"] = Value::from(uid);
            json_node.as_object_mut().unwrap().remove("node_key");

            info!("json_node: {}", json_node.to_string());
            mutation.set_json = json_node.to_string().into_bytes();

            info!("mutation with uid: {}", json_node);


            info!("Transaction: {:#?}", txn);
            client.mutate(&mutation)?;
            
            txn.set_commit_ts(SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?.as_secs());
            client.commit_or_abort(&txn)?;

            uid.to_string()
        }
        None => {
            // Add node
            let mut mutation = dgraph_client::api::Mutation::new();
            mutation.commit_now = true;
            mutation.set_json = (*node).clone().into_json().to_string().into_bytes();

            info!("mutation: {}", (*node).clone().into_json());
            txn.set_commit_ts(SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?.as_secs());

            let mut_res = client.mutate(&mutation);
            let uid = mut_res.map(|mut_res| {
                mut_res.get_uids().get("blank-0").unwrap().to_owned()
            })?;

            client.commit_or_abort(&txn)?;

            uid

        }
    };

    Ok(new_node_key)
}

pub fn set_process_schema(client: &DgraphClient) {
    let mut op_schema = dgraph_client::api::Operation::new();
    op_schema.schema = r#"
       		node_key: string @upsert @index(hash) .
       		pid: int @index(int) .
       		create_time: int @index(int) .
       		asset_id: string @index(hash) .
       		terminate_time: int @index(int) .
       		image_name: string @index(hash) .
       		arguments: string .
       		bin_file: uid @reverse .
       		children: uid @reverse .
       		created_files: uid @reverse .
            deleted_files: uid @reverse .
            read_files: uid @reverse .
            wrote_files: uid @reverse .
        "#.to_string();
    let _res = client.alter(&op_schema).expect("set schema");
}

pub fn set_file_schema(client: &DgraphClient) {
    let mut op_schema = dgraph_client::api::Operation::new();
    op_schema.schema = r#"
       		node_key: string @upsert @index(hash) .
       		asset_id: string @index(hash) .
       		create_time: int @index(int) .
       		delete_time: int @index(int) .
       		path: string @index(hash) .
        "#.to_string();
    let _res = client.alter(&op_schema).expect("set schema");
}

pub fn set_ip_address_schema(client: &mut DgraphClient) {
    let mut op_schema = dgraph_client::api::Operation::new();
    op_schema.schema = r#"
       		node_key: string @upsert @index(hash) .
       		last_seen: int @index(int) .
       		ip: string @index(hash) .
        "#.to_string();
    let _res = client.alter(&op_schema).expect("set schema");
}


