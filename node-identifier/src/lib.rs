#[macro_use]
extern crate mysql;

#[macro_use]
extern crate log;

#[macro_use]
extern crate failure;

extern crate stopwatch;
extern crate base58;
extern crate graph_descriptions;
extern crate sqs_microservice;
extern crate rusoto_core;
extern crate rusoto_s3;
extern crate uuid;
extern crate prost;
extern crate futures;
extern crate sha2;
extern crate lru_time_cache;
extern crate zstd;

use lru_time_cache::LruCache;


macro_rules! log_time {
    ($msg:expr, $x:expr) => {
        {
            let mut sw = Stopwatch::start_new();
            #[allow(path_statements)]
            let result = $x;
            sw.stop();
            info!("{} {} milliseconds", $msg, sw.elapsed_ms());
            result
        }
    };
}

pub mod ip_asset_history;
pub mod session_history;
pub mod cache;
pub mod session;

use base58::ToBase58;
use stopwatch::Stopwatch;
use failure::Error;

use sha2::{Digest, Sha256};

use sqs_microservice::handle_message;

use std::env;


use rusoto_s3::{S3, S3Client};
use prost::Message;

use mysql as my;

use futures::future::Future;


use session_history::map_session_ids_to_graph;
use ip_asset_history::map_asset_ids_to_graph;
use graph_descriptions::graph_description::*;
use graph_descriptions::*;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use rusoto_core::Region;
use sqs_microservice::handle_s3_sns_sqs_proto;
use std::collections::HashMap;
use std::collections::HashSet;
use std::time::Duration;
use std::sync::Arc;
use std::sync::Mutex;

use cache::IdentityCache;
use std::io::Cursor;

pub fn upload_identified_graphs(subgraph: GraphDescription) -> Result<(), Error> {
    info!("Uploading identified subgraphs");
    let s3 = S3Client::simple(
        Region::UsEast1
    );

    let subgraph: GraphDescription = subgraph.into();

    let mut body = Vec::with_capacity(5000);
    subgraph.encode(&mut body).expect("Failed to encode subgraph");

    let mut compressed = Vec::with_capacity(body.len());
    let mut proto = Cursor::new(&body);

    zstd::stream::copy_encode(&mut proto, &mut compressed, 4)
        .expect("compress zstd capnp");

    let mut hasher = Sha256::default();
    hasher.input(&body);

    let key = hasher.result().as_ref().to_base58();

    let bucket_prefix = std::env::var("BUCKET_PREFIX").expect("BUCKET_PREFIX");

    let bucket = bucket_prefix + "-subgraphs-generated-bucket";
    let epoch = SystemTime::now()
        .duration_since(UNIX_EPOCH).unwrap().as_secs();

    let day = epoch - (epoch % (24 * 60 * 60));

    let key = format!("{}/{}",
                      day,
                      key
    );
    info!("Uploading identified subgraphs to {}", key);
    s3.put_object(
        &rusoto_s3::PutObjectRequest {
            bucket,
            key: key.clone(),
            body: Some(compressed),
            ..Default::default()
        }
    ).wait()?;
    info!("Uploaded identified subgraphs to {}", key);

    Ok(())
}

pub fn identify_nodes(should_default: bool) {
    let max_count = 100_000;
    let time_to_live = Duration::from_secs(60 * 5);


    // TODO: Update the library to allow for customizing the hasher
    let username = env::var("HISTORY_DB_USERNAME").expect("IDENTITY_CACHE_PEPPER");
    let lru_cache = IdentityCache::new(max_count, time_to_live, b"pepper");


    handle_s3_sns_sqs_proto( move |mut subgraphs: GeneratedSubgraphs| {
        info!("Connecting to history database");

        let username = env::var("HISTORY_DB_USERNAME").expect("HISTORY_DB_USERNAME");
        let password = env::var("HISTORY_DB_PASSWORD").expect("HISTORY_DB_PASSWORD");

        let pool = my::Pool::new(
            format!("mysql://{username}:{password}@db.historydb:3306/historydb",
                    username=username,
                    password=password)
        ).unwrap();

        info!("Connected to history database");

        info!("Handling {} subgraphs", subgraphs.subgraphs.len());

        log_time!{
            "creating tables",
            {
                ip_asset_history::create_table(&pool);
                session_history::create_process_table(&pool);
                session_history::create_file_table(&pool);
                session_history::create_connection_table(&pool);
            }
        }

        subgraphs.subgraphs.sort_unstable_by(|a, b| a.timestamp.cmp(&b.timestamp));

        let mut result = Ok(());
        for unid_subgraph in subgraphs.subgraphs {
            let lru_cache = lru_cache.clone();
            let _result: Result<(), Error> = (|| {
                let mut output_subgraph = GraphDescription::new(unid_subgraph.timestamp);
                let mut unid_subgraph: GraphDescription = unid_subgraph.into();
                let mut result = Ok(());

                let mut unid_id_map = HashMap::new();
                let mut dead_node_ids = HashSet::new();

                info!("Mapping asset ids to graph");

                let r = map_asset_ids_to_graph(
                    &pool,
                    &mut dead_node_ids,
                    &mut unid_subgraph,
                );
                if let e @ Err(_) = r {
                    error!("error: {:#?}", e);
                    result = e;
                }

                info!("removing {} nodes and their edges", dead_node_ids.len());
                remove_dead_nodes(&dead_node_ids, &mut unid_subgraph);
                dead_node_ids.clear();

                info!("Mapping process session ids to graph");

                // Process/ File mapping *must* happen after asset ids
                let r = map_session_ids_to_graph(
                    &pool,
                    &mut unid_id_map,
                    &mut dead_node_ids,
                    &unid_subgraph,
                    &mut output_subgraph,
                    should_default,
                    lru_cache
                );

                if let e @ Err(_) = r {
                    error!("error: {:#?}", e);
                    result = e;
                }

                info!("{:#?}", unid_subgraph);

                log_time! {
                    "remap_edges",
                    remap_edges(&unid_id_map, &dead_node_ids, &unid_subgraph, &mut output_subgraph)
                }

//                lru_cache.check_cache("a");

                let r = upload_identified_graphs(output_subgraph);
                if let e @ Err(_) = r {
                    error!("error: {:#?}", e);
                    result = e;
                }


                result
            })();



            if let e @ Err(_) = _result {
                error!("error: {:#?}", e);
                result = e;
            }
        }

        result
    }, move |_| {Ok(())})


}


pub fn remove_dead_nodes(dead_node_ids: &HashSet<String>,
                         unid_subgraph: &mut GraphDescription) {
    for node_id in dead_node_ids.iter() {
        unid_subgraph.nodes.remove(node_id);

    }

    for (_node_key, edges) in unid_subgraph.edges.iter_mut() {
        let mut new_edges = vec![];
        for edge in &edges.edges {
            if dead_node_ids.contains(&edge.from_neighbor_key) ||
                dead_node_ids.contains(&edge.to_neighbor_key) {
                continue
            }
            new_edges.push(edge.clone());
        }
        edges.edges = new_edges;
    }

}



pub fn remap_edges(key_map: &HashMap<String, String>,
                    dead_node_ids: &HashSet<String>,
                    input_subgraph: &GraphDescription,
                    output_subgraph: &mut GraphDescription) {

    for (_node_key, edges) in &input_subgraph.edges {
        for edge in &edges.edges {

            if dead_node_ids.contains(&edge.from_neighbor_key) {
                warn!("Removing edge.from_neighbor_key {}", edge.from_neighbor_key);
                continue
            }

            if dead_node_ids.contains(&edge.to_neighbor_key) {
                warn!("Removing edge.to_neighbor_key {}", edge.to_neighbor_key);
                continue
            }

            let from_neighbor_key = key_map.get(&edge.from_neighbor_key)
                .expect("from_neighbor_key");
            let to_neighbor_key = key_map.get(&edge.to_neighbor_key)
                .expect("to_neighbor_key");

            output_subgraph.add_edge(
                edge.edge_name.to_owned(),
                from_neighbor_key.to_owned(),
                to_neighbor_key.to_owned(),
            )
        }
    }
}
