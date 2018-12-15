extern crate failure;
extern crate rayon;
extern crate log;
extern crate stopwatch;

extern crate chrono;
extern crate graph_generator_lib;
extern crate graph_descriptions;
extern crate regex;
extern crate sysmon;
extern crate sqs_microservice;

use rayon::prelude::*;
use rayon::iter::Either;
use log::*;
use failure::bail;
use failure::Error;
use chrono::prelude::*;
use sqs_microservice::handle_raw_event;
use graph_descriptions::*;
use graph_descriptions::graph_description::*;
use graph_generator_lib::upload_subgraphs;
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
    let dt = NaiveDateTime::parse_from_str(
        utc, "%Y-%m-%d %H:%M:%S%.3f")?;

    let dt: DateTime<Utc> = DateTime::from_utc(dt, Utc);
    let ts = dt.timestamp_millis();

    if ts < 0 {
        bail!("Timestamp is negative")
    }

    Ok(1)
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
        .timestamp(timestamp)
        .build()
        .unwrap();

    let child = ProcessDescriptionBuilder::default()
        .asset_id(process_start.header.computer.clone())
        .image_name(process_start.image.clone())
        .state(ProcessState::Created)
        .pid(process_start.process_id)
        .timestamp(timestamp)
        .build()
        .unwrap();

    let child_exe = FileDescriptionBuilder::default()
        .asset_id(process_start.header.computer.clone())
        .state(FileState::Existing)
        .timestamp(timestamp)
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
        .timestamp(timestamp)
        .build()
        .unwrap();

    let file = FileDescriptionBuilder::default()
        .asset_id(file_create.header.computer.clone())
        .state(FileState::Created)
        .path(file_create.target_filename)
        .timestamp(timestamp)
        .build()
        .unwrap();


    graph.add_edge("created_files",
                   creator.clone_key(),
                   file.clone_key());
    graph.add_node(creator);
    graph.add_node(file);

    Ok(graph)
}

use std::borrow::Cow;

fn main() {

    handle_raw_event(|event: Vec<u8>| {
        info!("Handling raw event");


        let events: Vec<_> = log_time!(
            "event split",
            event.split(|i| &[*i][..] == &b"\n"[..]).collect()
        );

        let subgraphs = log_time!(
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

        let graphs = GeneratedSubgraphs {subgraphs};
//
//        log_time!(
//            "upload_subgraphs",
//            upload_subgraphs(graphs)
//        )?;


        Ok(())
    });
}
