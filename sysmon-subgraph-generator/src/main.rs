extern crate aws_lambda_events;
extern crate chrono;
extern crate failure;
extern crate futures;
extern crate graph_descriptions;
extern crate graph_generator_lib;
extern crate lambda_runtime as lambda;
extern crate log;
extern crate openssl_probe;
extern crate rayon;
extern crate regex;
extern crate rusoto_core;
extern crate rusoto_s3;
extern crate rusoto_sqs;
extern crate serde;
extern crate simple_logger;
extern crate sqs_lambda;
extern crate stopwatch;
extern crate sysmon;


use std::borrow::Cow;
use std::marker::PhantomData;
use std::sync::Arc;

use aws_lambda_events::event::sqs::{SqsEvent, SqsMessage};
use chrono::prelude::*;
use failure::bail;
use failure::Error;
use futures::{Future, Stream};
use graph_descriptions::*;
use graph_descriptions::graph_description::*;
use graph_generator_lib::upload_subgraphs;
use lambda::Context;
use lambda::error::HandlerError;
use lambda::Handler;
use lambda::lambda;
use log::*;
use log::error;
use rayon::iter::Either;
use rayon::prelude::*;
use rusoto_core::Region;
use rusoto_s3::{GetObjectRequest, S3};
use rusoto_s3::S3Client;
use rusoto_sqs::{GetQueueUrlRequest, Sqs, SqsClient};
use serde::Deserialize;
use sqs_lambda::BlockingSqsCompletionHandler;
use sqs_lambda::EventHandler;
use sqs_lambda::events_from_s3_sns_sqs;
use sqs_lambda::S3EventRetriever;
use sqs_lambda::SqsService;
use sqs_lambda::ZstdDecoder;
use sysmon::*;

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

pub fn utc_to_epoch(utc: &str) -> Result<u64, Error> {
    let utc_time = "2017-04-28 22:08:22.025";

    let dt = NaiveDateTime::parse_from_str(
        utc, "%Y-%m-%d %H:%M:%S%.3f")?;

    let dt: DateTime<Utc> = DateTime::from_utc(dt, Utc);
    let ts = dt.timestamp_millis();

    if ts < 0 {
        bail!("Timestamp is negative")
    }

    Ok(ts as u64)
}

fn handle_process_start(process_start: ProcessCreateEvent) -> Result<GraphDescription, Error> {
    let timestamp = utc_to_epoch(&process_start.utc_time)?;
    let mut graph = GraphDescription::new(
        timestamp
    );

    let parent = ProcessDescriptionBuilder::default()
        .asset_id(process_start.header.computer.clone())
        .state(ProcessState::Existing)
        .pid(process_start.parent_process_id)
        .last_seen_timestamp(timestamp)
        .build()
        .unwrap();

    let child = ProcessDescriptionBuilder::default()
        .asset_id(process_start.header.computer.clone())
        .image_name(process_start.image.clone())
        .state(ProcessState::Created)
        .pid(process_start.process_id)
        .created_timestamp(timestamp)
        .build()
        .unwrap();

    let child_exe = FileDescriptionBuilder::default()
        .asset_id(process_start.header.computer.clone())
        .state(FileState::Existing)
        .last_seen_timestamp(timestamp)
        .path(process_start.image)
        .build()
        .unwrap();

        graph.add_edge("bin_file",
                       child.clone_key(),
                       child_exe.clone_key()
        );

    graph.add_node(child_exe);

    graph.add_edge("children",
                   parent.clone_key(),
                   child.clone_key());
    graph.add_node(parent);
    graph.add_node(child);

    Ok(graph)
}

fn handle_file_create(file_create: FileCreateEvent) -> Result<GraphDescription, Error> {
    let timestamp = utc_to_epoch(&file_create.creation_utc_time)?;
    let mut graph = GraphDescription::new(
        timestamp
    );

    let creator = ProcessDescriptionBuilder::default()
        .asset_id(file_create.header.computer.clone())
        .state(ProcessState::Existing)
        .pid(file_create.process_id)
        .last_seen_timestamp(timestamp)
        .build()
        .unwrap();

    let file = FileDescriptionBuilder::default()
        .asset_id(file_create.header.computer.clone())
        .state(FileState::Created)
        .path(file_create.target_filename)
        .created_timestamp(timestamp)
        .build()
        .unwrap();


    graph.add_edge("created_files",
                   creator.clone_key(),
                   file.clone_key());
    graph.add_node(creator);
    graph.add_node(file);

    Ok(graph)
}

#[derive(Clone)]
struct SysmonSubgraphGenerator;

impl EventHandler<Vec<u8>> for SysmonSubgraphGenerator {
    fn handle_event(&self, event: Vec<u8>) -> Result<(), Error> {
        info!("Handling raw event");


        let events: Vec<_> = log_time!(
            "event split",
            event.split(|i| &[*i][..] == &b"\n"[..]).collect()
        );

        let subgraphs: Vec<_> = log_time!(
            "events par_iter",
             events.into_par_iter().flat_map(move |event| {
                let event = String::from_utf8_lossy(event);
                let event = Event::from_str(&event);
                let event = event.ok()?;

                match event {
                    Event::ProcessCreate(event) => {
                        match handle_process_start(event) {
                            Ok(event) => Some(event),
                            Err(e) => {
                                warn!("Failed to process event: {}", e);
                                None
                            }
                        }
                    }
                    Event::FileCreate(event) => {
                        match handle_file_create(event) {
                            Ok(event) => Some(event),
                            Err(e) => {
                                warn!("Failed to process event: {}", e);
                                None
                            }
                        }
                    }
                    Event::InboundNetwork(event) => {
                        warn!("Network traffic not yet supported");
                        None
                    }
                    Event::OutboundNetwork(event) => {
                        warn!("Network traffic not yet supported");
                        None
                    }
                }
            }).collect()
        );

        info!("Completed mapping {} subgraphs", subgraphs.len());
        let graphs = GeneratedSubgraphs {subgraphs};

        log_time!(
            "upload_subgraphs",
            upload_subgraphs(graphs)
        )?;


        Ok(())
    }
}


fn my_handler(event: SqsEvent, ctx: Context) -> Result<(), HandlerError> {
    let region = Region::UsEast1;
    info!("Creating sqs_client");
    let sqs_client = Arc::new(SqsClient::simple(region.clone()));

    info!("Creating s3_client");
    let s3_client = Arc::new(S3Client::simple(region.clone()));

    info!("Creating retriever");
    let retriever = S3EventRetriever::new(
        s3_client,
        |d| {info!("Parsing: {:?}", d); events_from_s3_sns_sqs(d)},
        ZstdDecoder{buffer: Vec::with_capacity(1 << 8)},
    );

    let queue_url = std::env::var("QUEUE_URL").expect("QUEUE_URL");

    info!("Creating sqs_completion_handler");
    let sqs_completion_handler = BlockingSqsCompletionHandler::new(
        sqs_client,
        queue_url
    );

    let handler = SysmonSubgraphGenerator{};

    let mut sqs_service = SqsService::new(
        retriever,
        handler,
        sqs_completion_handler,
    );

    info!("Handing off event");
    sqs_service.run(event, ctx)?;

    Ok(())
}

fn main()  -> Result<(), Box<dyn std::error::Error>> {
    openssl_probe::init_ssl_cert_env_vars();
    simple_logger::init_with_level(log::Level::Info).unwrap();

    info!("Starting lambda");
    lambda!(my_handler);
    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_time() {
        let utc_time = "2017-04-28 22:08:22.025";
        let ts = utc_to_epoch(utc_time).unwrap();

    }

}