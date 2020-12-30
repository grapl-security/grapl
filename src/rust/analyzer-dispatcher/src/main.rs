#![type_length_limit = "1214269"]
// Our types are simply too powerful

use std::collections::HashSet;
use std::convert::TryInto;
use std::io::{Cursor, Stdout};
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use aws_lambda_events::event::s3::{
    S3Bucket, S3Entity, S3Event, S3EventRecord, S3Object, S3RequestParameters, S3UserIdentity,
};
use aws_lambda_events::event::sqs::SqsEvent;
use bytes::Bytes;
use chrono::Utc;
use failure::{bail, Error};
use log::{debug, error, info, warn};
use prost::Message;
use rusoto_core::{HttpClient, Region};
use rusoto_s3::{ListObjectsRequest, S3Client, S3};
use rusoto_sqs::{SendMessageRequest, Sqs, SqsClient};
use serde::{Deserialize, Serialize};
use serde_json::json;

use grapl_graph_descriptions::graph_description::*;
use grapl_observe::metric_reporter::MetricReporter;
use sqs_executor::cache::NopCache;
use sqs_executor::completion_event_serializer::CompletionEventSerializer;
use sqs_executor::errors::{CheckedError, Recoverable};
use sqs_executor::event_decoder::PayloadDecoder;
use sqs_executor::event_handler::{CompletedEvents, EventHandler};
use sqs_executor::event_retriever::S3PayloadRetriever;
use sqs_executor::s3_event_emitter::S3EventEmitter;
use sqs_executor::{make_ten, time_based_key_fn};

#[derive(Debug)]
pub struct AnalyzerDispatcher<S>
where
    S: S3 + Send + Sync + 'static,
{
    s3_client: Arc<S>,
}

impl<S> Clone for AnalyzerDispatcher<S>
where
    S: S3 + Send + Sync + 'static,
{
    fn clone(&self) -> Self {
        Self {
            s3_client: self.s3_client.clone(),
        }
    }
}

async fn get_s3_keys(
    s3_client: &impl S3,
    bucket: impl Into<String>,
) -> Result<impl IntoIterator<Item = Result<String, Error>>, Error> {
    let bucket = bucket.into();

    let list_res = tokio::time::timeout(
        Duration::from_secs(2),
        s3_client.list_objects(ListObjectsRequest {
            bucket,
            ..Default::default()
        }),
    )
    .await??;

    let contents = match list_res.contents {
        Some(contents) => contents,
        None => {
            warn!("List response returned nothing");
            Vec::new()
        }
    };

    Ok(contents.into_iter().map(|object| match object.key {
        Some(key) => Ok(key),
        None => bail!("S3Object is missing key"),
    }))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzerDispatchEvent {
    key: String,
    subgraph: Graph,
}

fn encode_subgraph(subgraph: &Graph) -> Result<Vec<u8>, Error> {
    let mut buf = Vec::with_capacity(5000);
    subgraph.encode(&mut buf)?;
    Ok(buf)
}

#[derive(thiserror::Error, Debug)]
pub enum AnalyzerDispatcherError {
    #[error("Unexpected")]
    Unexpected(String),
}

impl CheckedError for AnalyzerDispatcherError {
    fn error_type(&self) -> Recoverable {
        Recoverable::Transient
    }
}

#[async_trait]
impl<S> EventHandler for AnalyzerDispatcher<S>
where
    S: S3 + Send + Sync + 'static,
{
    type InputEvent = GeneratedSubgraphs;
    type OutputEvent = Vec<AnalyzerDispatchEvent>;
    type Error = AnalyzerDispatcherError;

    async fn handle_event(
        &mut self,
        subgraphs: Self::InputEvent,
        completed: &mut CompletedEvents,
    ) -> Result<Self::OutputEvent, Result<(Self::OutputEvent, Self::Error), Self::Error>> {
        let bucket = std::env::var("BUCKET_PREFIX")
            .map(|prefix| format!("{}-analyzers-bucket", prefix))
            .expect("BUCKET_PREFIX");

        let mut subgraph = Graph::new(0);

        subgraphs
            .subgraphs
            .iter()
            .for_each(|generated_subgraph| subgraph.merge(generated_subgraph));

        if subgraph.is_empty() {
            warn!("Attempted to handle empty subgraph");
            return Ok(vec![]);
        }

        info!("Retrieving S3 keys");
        let keys = match get_s3_keys(self.s3_client.as_ref(), &bucket).await {
            Ok(keys) => keys,
            Err(e) => {
                return Err(Err(AnalyzerDispatcherError::Unexpected(format!(
                    "Failed to list bucket: {} with {:?}",
                    bucket, e
                ))));
            }
        };

        let mut dispatch_events = Vec::new();

        let mut failed = None;
        for key in keys {
            let key = match key {
                Ok(key) => key,
                Err(e) => {
                    warn!("Failed to retrieve key with {:?}", e);
                    failed = Some(e);
                    continue;
                }
            };

            dispatch_events.push(AnalyzerDispatchEvent {
                key,
                subgraph: subgraph.clone(),
            });
        }

        if let Some(e) = failed {
            Err(Ok((
                dispatch_events,
                AnalyzerDispatcherError::Unexpected(e.to_string()),
            )))
        } else {
            Ok(dispatch_events)
        }

        // identities.into_iter().for_each(|identity| completed.add_identity(identity));
    }
}

#[derive(Clone, Debug, Default)]
pub struct SubgraphSerializer {
    proto: Vec<u8>,
}

#[derive(thiserror::Error, Debug)]
pub enum SubgraphSerializerError {
    #[error("IO")]
    Io(#[from] std::io::Error),
    #[error("EncodeError")]
    JsonEncodeError(#[from] serde_json::Error),
    #[error("ProtoEncodeError")]
    ProtoEncodeError(Error),
}

impl CompletionEventSerializer for SubgraphSerializer {
    type CompletedEvent = Vec<AnalyzerDispatchEvent>;
    type Output = Vec<u8>;
    type Error = SubgraphSerializerError;

    fn serialize_completed_events(
        &mut self,
        completed_events: &[Self::CompletedEvent],
    ) -> Result<Vec<Self::Output>, Self::Error> {
        let unique_events: Vec<_> = completed_events.iter().flatten().collect();

        let mut final_subgraph = Graph::new(0);

        for event in unique_events.iter() {
            final_subgraph.merge(&event.subgraph);
        }

        let mut serialized = Vec::with_capacity(unique_events.len());

        for event in unique_events {
            let event = json!({
                "key": event.key,
                "subgraph": encode_subgraph(&final_subgraph).map_err(|e| SubgraphSerializerError::ProtoEncodeError(e))?
            });

            serialized.push(serde_json::to_vec(&event)?);
        }

        Ok(serialized)
    }
}

#[derive(Debug, Clone, Default)]
pub struct ZstdProtoDecoder;

impl<E> PayloadDecoder<E> for ZstdProtoDecoder
where
    E: Message + Default,
{
    fn decode(&mut self, body: Vec<u8>) -> Result<E, Box<dyn std::error::Error>>
    where
        E: Message + Default,
    {
        let mut decompressed = Vec::new();

        let mut body = Cursor::new(&body);

        zstd::stream::copy_decode(&mut body, &mut decompressed)?;

        let buf = Bytes::from(decompressed);

        Ok(E::decode(buf)?)
    }
}

async fn handler() -> Result<(), Box<dyn std::error::Error>> {
    let env = grapl_config::init_grapl_env!();

    info!("Handling event");

    let sqs_client = SqsClient::new(grapl_config::region());
    let s3_client = S3Client::new(grapl_config::region());
    let source_queue_url = std::env::var("SOURCE_QUEUE_URL").expect("SOURCE_QUEUE_URL");
    debug!("Queue Url: {}", source_queue_url);
    let bucket_prefix = std::env::var("BUCKET_PREFIX").expect("BUCKET_PREFIX");

    let destination_bucket = bucket_prefix + "-dispatched-analyzer-bucket";
    info!("Output events to: {}", destination_bucket);
    let region = grapl_config::region();

    let cache = &mut make_ten(async {
        NopCache {}
        // RedisCache::new(cache_address.to_owned(), MetricReporter::<Stdout>::new(&env.service_name)).await
        //     .expect("Could not create redis client")
    })
    .await;

    let serializer = &mut make_ten(async { SubgraphSerializer::default() }).await;

    let s3_emitter = &mut make_ten(async {
        S3EventEmitter::new(
            s3_client.clone(),
            destination_bucket.clone(),
            time_based_key_fn,
            MetricReporter::new(&env.service_name),
        )
    })
    .await;

    let s3_payload_retriever = &mut make_ten(async {
        S3PayloadRetriever::new(
            |region_str| S3Client::new(Region::from_str(&region_str).expect("region_str")),
            ZstdProtoDecoder::default(),
            MetricReporter::new(&env.service_name),
        )
    })
    .await;
    let analyzer_dispatcher = &mut make_ten(async {
        AnalyzerDispatcher {
            s3_client: Arc::new(S3Client::new(region.clone())),
        }
    })
    .await;

    info!("Starting process_loop");
    sqs_executor::process_loop(
        std::env::var("QUEUE_URL").expect("QUEUE_URL"),
        std::env::var("DEAD_LETTER_QUEUE_URL").expect("DEAD_LETTER_QUEUE_URL"),
        cache,
        sqs_client.clone(),
        analyzer_dispatcher,
        s3_payload_retriever,
        s3_emitter,
        serializer,
        MetricReporter::new(&env.service_name),
    )
    .await;

    info!("Exiting");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    handler().await?;
    Ok(())
}
