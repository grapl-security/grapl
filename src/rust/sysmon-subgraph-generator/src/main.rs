#![type_length_limit = "1334469"]

mod metrics;
mod models;
mod serialization;
mod generator;

use std::collections::HashSet;
use std::fmt::Debug;
use std::io::Cursor;
use std::marker::PhantomData;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use aws_lambda_events::event::s3::{
    S3Bucket, S3Entity, S3Event, S3EventRecord, S3Object, S3RequestParameters, S3UserIdentity,
};
use aws_lambda_events::event::sqs::SqsEvent;
use chrono::prelude::*;
use failure::bail;
use failure::Error;
use lambda_runtime::error::HandlerError;
use lambda_runtime::lambda;
use lambda_runtime::Context;
use rusoto_core::credential::AwsCredentials;
use rusoto_core::{HttpClient, Region, RusotoError};
use rusoto_s3::GetObjectRequest;
use rusoto_s3::{S3Client, S3};
use rusoto_sqs::GetQueueUrlRequest;
use rusoto_sqs::{ListQueuesRequest, SendMessageRequest, Sqs, SqsClient};
use serde::Deserialize;
use sqs_lambda::cache::Cache;
use sqs_lambda::cache::{CacheResponse, NopCache};
use sqs_lambda::completion_event_serializer::CompletionEventSerializer;
use sqs_lambda::event_decoder::PayloadDecoder;
use sqs_lambda::event_handler::{Completion, EventHandler, OutputEvent};
use sqs_lambda::event_processor::{EventProcessor, EventProcessorActor};
use sqs_lambda::event_retriever::S3PayloadRetriever;
use sqs_lambda::local_sqs_service::local_sqs_service;
use sqs_lambda::redis_cache::RedisCache;
use sqs_lambda::s3_event_emitter::S3EventEmitter;
use sqs_lambda::sqs_completion_handler::{
    CompletionPolicy, SqsCompletionHandler, SqsCompletionHandlerActor,
};
use sqs_lambda::sqs_consumer::{ConsumePolicy, SqsConsumer, SqsConsumerActor};
use sqs_lambda::sqs_service::sqs_service;
use sysmon::*;
use tokio::runtime::Runtime;
use uuid::Uuid;

use async_trait::async_trait;
use graph_descriptions::file::FileState;
use graph_descriptions::process::ProcessState;
use graph_descriptions::graph_description::*;
use graph_descriptions::network_connection::NetworkConnectionState;
use graph_descriptions::process_inbound_connection::ProcessInboundConnectionState;
use graph_descriptions::process_outbound_connection::ProcessOutboundConnectionState;
use lazy_static::lazy_static;

use graph_generator_lib::*;

use graph_descriptions::node::NodeT;
use log::*;

use crate::metrics::SysmonSubgraphGeneratorMetrics;
use grapl_config::*;
use crate::serialization::ZstdDecoder;
use crate::generator::SysmonSubgraphGenerator;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let env = grapl_config::init_grapl_env!();
    info!("Starting sysmon-subgraph-generator");

    let metrics = SysmonSubgraphGeneratorMetrics::new(&env.service_name);

    if grapl_config::is_local() {
        let generator = SysmonSubgraphGenerator::new(NopCache {}, metrics);

        run_graph_generator(generator, ZstdDecoder::default()).await;
    } else {
        let generator = SysmonSubgraphGenerator::new(event_cache().await, metrics);

        run_graph_generator(generator, ZstdDecoder::default()).await;
    }

    Ok(())
}
