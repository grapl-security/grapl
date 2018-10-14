#![feature(nll)]

#[macro_use]
extern crate prost_derive;
#[macro_use] extern crate log;

extern crate base64;
extern crate failure;
extern crate mysql;
extern crate sqs_microservice;
extern crate graph_descriptions;
extern crate rusoto_core;
extern crate rusoto_s3;
extern crate uuid;
extern crate prost;
extern crate futures_await as futures;
extern crate sha2;

use failure::Error;

use sha2::{Digest, Sha256};

use sqs_microservice::handle_s3_sns_sqs_proto;
use postgres::{Connection, TlsMode};

use rusoto_s3::{S3, S3Client};
use prost::Message;

use futures::future::Future;

mod ip_asset_mapper;

use rusoto_core::Region;

use ip_asset_mapping::{IpAssetMapping, IpAssetMappings};
use ip_asset_mapper::create_ip_asset_session;
use std::env;

mod ip_asset_mapping {
    include!(concat!(env!("OUT_DIR"), "/ip_asset_mapping.rs"));
}

fn main() {

    handle_s3_sns_sqs_proto(move |ip_asset_mappings: IpAssetMappings| {
        info!("Attempting to connect to postgres");

        let username = env::var("HISTORY_DB_USERNAME").expect("HISTORY_DB_USERNAME");
        let password = env::var("HISTORY_DB_PASSWORD").expect("HISTORY_DB_PASSWORD");

        let pool = my::Pool::new(
            format!("mysql://{username}:{password}@db.historydb:3306/historydb",
                    username=username,
                    password=password)
        ).unwrap();

        info!("Connected successfully to postgres");

        for ip_asset_mapping in ip_asset_mappings.mappings {
            let ip = ip_asset_mapping.ip;
            let asset_id = ip_asset_mapping.asset_id;
            let timestamp = ip_asset_mapping.timestamp;
            info!("Mapping ip {} timestamp {} to assset_id{}",
                    ip, timestamp, asset_id);

            create_ip_asset_session(
                &conn,
                ip,
                asset_id,
                timestamp
            )?;
        }

        Ok(())
    }, move |_| {Ok(())})
}


