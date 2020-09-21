#![type_length_limit = "1195029"]

mod aws;
mod local;
mod serialization;

use std::collections::HashSet;
use std::io::Cursor;
use std::sync::Arc;
use std::time::Duration;

use std::fmt::Debug;

use log::*;
use rusoto_core::{HttpClient, Region, RusotoResult};
use rusoto_s3::S3Client;
use rusoto_sqs::{SendMessageError, SendMessageRequest, Sqs, SqsClient};
use rusoto_sts::{StsAssumeRoleSessionCredentialsProvider, StsClient};
use serde::Deserialize;

use aws_lambda_events::event::s3::{
    S3Bucket, S3Entity, S3Event, S3EventRecord, S3Object, S3RequestParameters, S3UserIdentity,
};
use aws_lambda_events::event::sqs::SqsEvent;
use chrono::Utc;
use graph_descriptions::graph_description::*;
use grapl_config as config;
use lambda_runtime::error::HandlerError;
use lambda_runtime::Context;
use sqs_lambda::cache::NopCache;
use sqs_lambda::completion_event_serializer::CompletionEventSerializer;
use sqs_lambda::event_decoder::PayloadDecoder;
use sqs_lambda::event_handler::EventHandler;
use sqs_lambda::local_sqs_service::local_sqs_service;
use std::str::FromStr;

/// Graph generator implementations should invoke this function to begin processing new log events.
///
/// ```
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     use sqs_lambda::cache::NopCache;
///     use graph_generator_lib::run_graph_generator;
///
///     grapl_config::init_grapl_log!();
///
///     if grapl_config::is_local() {
///         run_graph_generator(
///             MyNewGenerator::new(NopCache {}),
///             MyDecoder::default()
///         ).await?;
///     } else {
///         run_graph_generator(
///             MyNewGenerator::new(grapl_config::event_cache().await),
///             MyDecoder::default()
///         ).await?;
///     }
///
///     Ok(())
/// }
/// ```
pub async fn run_graph_generator<
    IE: Send + Sync + Clone + 'static,
    EH: EventHandler<InputEvent = IE, OutputEvent = Graph, Error = sqs_lambda::error::Error>
        + Send
        + Sync
        + Clone
        + 'static,
    ED: PayloadDecoder<IE> + Send + Sync + Clone + 'static,
>(
    generator: EH,
    event_decoder: ED,
) {
    info!("IS_LOCAL={:?}", config::is_local());

    if config::is_local() {
        local::run_graph_generator_local(generator, event_decoder).await;
    } else {
        aws::run_graph_generator_aws(generator, event_decoder);
    }
}
