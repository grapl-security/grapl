extern crate aws_lambda_events;
extern crate base64;
extern crate failure;
extern crate futures;
extern crate graph_descriptions;
extern crate lambda_runtime as lambda;
#[macro_use] extern crate log;
#[macro_use] extern crate mysql;

extern crate prost;
#[macro_use] extern crate prost_derive;
extern crate rusoto_core;
extern crate rusoto_s3;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate sha2;
extern crate simple_logger;
extern crate stopwatch;
extern crate uuid;
extern crate sqs_lambda;

use std::str::FromStr;
use std::env;

use aws_lambda_events::event::sqs::{SqsEvent, SqsMessage};
use failure::Error;
use futures::future::Future;
use lambda::Context;
use lambda::error::HandlerError;
use lambda::lambda;
use mysql::{Pool, Transaction};
use prost::Message;
use rusoto_core::Region;
use rusoto_s3::{S3, S3Client};
use sha2::{Digest, Sha256};
use sqs_lambda::EventHandler;
use sqs_lambda::events_from_s3_sns_sqs;
use sqs_lambda::NopSqsCompletionHandler;
use sqs_lambda::S3EventRetriever;
use sqs_lambda::SqsService;
use sqs_lambda::ZstdProtoDecoder;

use ip_asset_mapper::create_ip_asset_session;
use std::sync::Arc;
use sqs_lambda::ZstdJsonDecoder;
use hostname_asset_mapper::create_hostname_asset_session;

mod ip_asset_mapper;
mod hostname_asset_mapper;

macro_rules! log_time {
    ($msg:expr, $x:expr) => {
        {
            let mut sw = stopwatch::Stopwatch::start_new();
            #[allow(path_statements)]
            let result = $x;
            sw.stop();
            info!("{} {} milliseconds", $msg, sw.elapsed_ms());
            result
        }
    };
}

#[derive(Serialize, Deserialize)]
struct IpAssetMapping {
    ip: String,
    asset_id: String,
    timestamp: u64
}

#[derive(Serialize, Deserialize)]
struct HostnameAssetMapping {
    hostname: String,
    asset_id: String,
    timestamp: u64
}

#[derive(Serialize, Deserialize)]
enum Mapping {
    IpAsset(IpAssetMapping),
    HostnameAsset(HostnameAssetMapping)
}

#[derive(Debug, Clone)]
struct NodeIdentityMapper;

impl EventHandler<Vec<Mapping>> for NodeIdentityMapper {
    fn handle_event(&self, asset_mappings: Vec<Mapping>) -> Result<(), Error> {
        let username = env::var("HISTORY_DB_USERNAME").expect("HISTORY_DB_USERNAME");
        let password = env::var("HISTORY_DB_PASSWORD").expect("HISTORY_DB_PASSWORD");

        info!("Attempting to connect to mysql");

        let pool = mysql::Pool::new(
            format!("mysql://{username}:{password}@db.historydb:3306/historydb",
                    username=username,
                    password=password)
        ).expect("Failed to connect to historydb");

        info!("Connected successfully to mysql");

        ip_asset_mapper::create_table(&pool);
        hostname_asset_mapper::create_table(&pool);

        for asset_mapping in asset_mappings {
            match asset_mapping {
                Mapping::IpAsset(ip_asset_mapping) => {
                    create_ip_asset_session(
                        &pool,
                        ip_asset_mapping.ip,
                        ip_asset_mapping.asset_id,
                        ip_asset_mapping.timestamp,
                    )?;
                },
                Mapping::HostnameAsset(host_asset_mapping) => {
                    create_hostname_asset_session(
                        &pool,
                        host_asset_mapping.hostname,
                        host_asset_mapping.asset_id,
                        host_asset_mapping.timestamp,
                    )?;
                }
            }
        }

        Ok(())

    }
}

pub fn handler(event: SqsEvent, ctx: Context) -> Result<(), HandlerError> {
    let handler = NodeIdentityMapper{};
    let region = {
        let region_str = env::var("AWS_REGION").expect("AWS_REGION");
        Region::from_str(&region_str).expect("Invalid Region")
    };

    info!("Creating s3_client");
    let s3_client = Arc::new(S3Client::new(region.clone()));

    info!("Creating retriever");
    let retriever = S3EventRetriever::new(
        s3_client,
        |d| {info!("Parsing: {:?}", d); events_from_s3_sns_sqs(d)},
        ZstdJsonDecoder{ buffer: Vec::with_capacity(512) },
    );

    let queue_url = std::env::var("QUEUE_URL").expect("QUEUE_URL");

    info!("Creating sqs_completion_handler");
    let sqs_completion_handler = NopSqsCompletionHandler::new(
        queue_url
    );

    let mut sqs_service = SqsService::new(
        retriever,
        handler,
        sqs_completion_handler,
    );

    info!("Handing off event");
    log_time!("sqs_service.run", sqs_service.run(event, ctx)?);

    Ok(())
}



fn main() {
    simple_logger::init_with_level(log::Level::Info).unwrap();
    lambda!(handler);
}


