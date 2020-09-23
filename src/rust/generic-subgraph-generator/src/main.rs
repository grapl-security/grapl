#![type_length_limit="1232619"]

mod models;
mod generator;
mod serialization;

use std::fmt::Debug;
use std::io::Cursor;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use aws_lambda_events::event::sqs::SqsEvent;
use regex::Regex;
use serde::de::Error as SerdeError;
use serde::{Deserialize, Serialize};
use sqs_lambda::cache::{Cache, CacheResponse, NopCache, Cacheable};
use sqs_lambda::event_decoder::PayloadDecoder;
use sqs_lambda::event_handler::{Completion, EventHandler, OutputEvent};
use tracing::*;

use async_trait::async_trait;
use graph_descriptions::file::FileState;
use graph_descriptions::graph_description::*;
use graph_descriptions::node::NodeT;
use graph_descriptions::process::ProcessState;
use graph_generator_lib::run_graph_generator;
use grapl_config::event_cache;
use lazy_static::lazy_static;
use tracing_subscriber::EnvFilter;
use crate::serialization::ZstdJsonDecoder;
use crate::generator::GenericSubgraphGenerator;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    grapl_config::init_grapl_env!();

    info!("Starting generic-subgraph-generator");

    if grapl_config::is_local() {
        let generator = GenericSubgraphGenerator::new(NopCache {});

        run_graph_generator(generator, ZstdJsonDecoder::default()).await;
    } else {
        let generator = GenericSubgraphGenerator::new(event_cache().await);

        run_graph_generator(generator, ZstdJsonDecoder::default()).await;
    }

    Ok(())
}