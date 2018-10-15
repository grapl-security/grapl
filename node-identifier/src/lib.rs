#![feature(nll)]

#[macro_use]
extern crate mysql;

#[macro_use]
extern crate log;

#[macro_use]
extern crate failure;

extern crate base58;
extern crate graph_descriptions;
extern crate sqs_microservice;
extern crate rusoto_core;
extern crate rusoto_s3;
extern crate uuid;
extern crate prost;
extern crate futures_await as futures;
extern crate sha2;

pub mod ip_asset_history;
pub mod file_history;
pub mod process_history;

use base58::ToBase58;

use failure::Error;

use sha2::{Digest, Sha256};

use sqs_microservice::handle_message;

use std::env;


use rusoto_s3::{S3, S3Client};
use prost::Message;

use mysql as my;

use futures::future::Future;


use process_history::map_process_session_ids_to_graph;
use ip_asset_history::map_asset_ids_to_graph;
use graph_descriptions::graph_description::*;
use graph_descriptions::*;
use file_history::map_file_session_ids_to_graph;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use rusoto_core::Region;
use sqs_microservice::handle_s3_sns_sqs_proto;


pub fn upload_identified_graphs(subgraph: GraphDescription) -> Result<(), Error> {
    info!("Uploading identified subgraphs");
    let s3 = S3Client::simple(
        Region::UsEast1
    );


    let subgraph: GraphDescriptionProto = subgraph.into();

    let mut body = Vec::new();
    subgraph.encode(&mut body).expect("Failed to encode subgraph");

    let mut hasher = Sha256::default();
    hasher.input(&body);

    let key = hasher.result().as_ref().to_base58();

    let bucket_prefix = std::env::var("BUCKET_PREFIX").expect("BUCKET_PREFIX");

    let bucket = bucket_prefix + "-subgraphs-generated-bucket";
    let epoch = SystemTime::now()
        .duration_since(UNIX_EPOCH).unwrap().as_secs();

    //
    // by day
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
            body: Some(body),
            ..Default::default()
        }
    ).wait()?;
    info!("Uploaded identified subgraphs to {}", key);

    Ok(())
}

pub fn identify_nodes(should_default: bool) {


    handle_s3_sns_sqs_proto( move |mut subgraphs: GeneratedSubgraphsProto| {
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

        ip_asset_history::create_table(&pool);
        process_history::create_table(&pool);
        file_history::create_table(&pool);

        subgraphs.subgraphs.sort_unstable_by(|a, b| a.timestamp.cmp(&b.timestamp));

        let mut result = Ok(());
        for subgraph in subgraphs.subgraphs {
            let _result: Result<(), Error> = (|| {
                let mut subgraph: GraphDescription = subgraph.into();
                let mut result = Ok(());

                info!("Mapping asset ids to graph");
                let mut p_transaction = pool.start_transaction(
                    false,
                    Some(my::IsolationLevel::Serializable),
                    Some(false)
                )?;

                let r = map_asset_ids_to_graph(&mut p_transaction, &mut subgraph);
                if let e @ Err(_) = r {
                    error!("error: {:#?}", e);
                    result = e;
                    p_transaction.rollback().expect("rollback");
                } else {
                    if let Err(e) = p_transaction.commit() {error!("{}", e)}
                }
                info!("Mapping process session ids to graph");

                // Process/ File mapping *must* happen after asset ids
                let mut p_transaction = pool.start_transaction(
                    false,
                    Some(my::IsolationLevel::Serializable),
                    Some(false)
                )?;

                let r = map_process_session_ids_to_graph(&mut p_transaction, &mut subgraph, should_default);
                if let e @ Err(_) = r {
                    error!("error: {:#?}", e);
                    result = e;
                    p_transaction.rollback().expect("rollback");
                } else {
                    if let Err(e) = p_transaction.commit() {error!("{}", e)}
                }

                let mut p_transaction = pool.start_transaction(
                    false,
                    Some(my::IsolationLevel::Serializable),
                    Some(false)
                )?;

                info!("Mapping file session ids to graph");
                let r = map_file_session_ids_to_graph(&mut p_transaction, &mut subgraph, should_default);
                if let e @ Err(_) = r {
                    error!("error: {:#?}", e);
                    result = e;
                    p_transaction.rollback().expect("rollback");
                } else {
                    if let Err(e) = p_transaction.commit() {error!("{}", e)}
                }

                info!("{:#?}", subgraph);

                let r = upload_identified_graphs(subgraph);
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