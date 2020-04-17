extern crate aws_lambda_events;
extern crate zstd;
extern crate base16;
extern crate base64;
extern crate dgraph_rs;
#[macro_use] extern crate failure;
extern crate futures;
extern crate graph_descriptions;
extern crate grpc;
extern crate lambda_runtime as lambda;
#[macro_use] extern crate log;

extern crate prost;
extern crate rusoto_core;
extern crate rusoto_s3;
extern crate rusoto_sns;
extern crate rusoto_sqs;
extern crate serde;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate serde_json;
extern crate sha2;
extern crate simple_logger;
extern crate sqs_lambda;
extern crate stopwatch;

use std::str::FromStr;

use std::collections::{HashMap, HashSet};
use std::io::Cursor;
use std::sync::Arc;
use std::time::Duration;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use aws_lambda_events::event::sqs::{SqsEvent, SqsMessage};
use dgraph_rs::protos::api;
use dgraph_rs::protos::api_grpc::{Dgraph, DgraphClient};
use dgraph_rs::protos::api_grpc;
use dgraph_rs::Transaction;
use failure::Error;
use futures::Future;
use graph_descriptions::graph_description::*;
use grpc::{Client, ClientStub};
use grpc::ClientConf;
use lambda::Context;
use lambda::error::HandlerError;
use lambda::lambda;
use prost::Message;
use rusoto_core::{Region, HttpClient};
use rusoto_s3::{ListObjectsRequest, PutObjectRequest, S3, S3Client};
use rusoto_sns::{Sns, SnsClient};
use rusoto_sns::PublishInput;
use rusoto_sqs::{GetQueueUrlRequest, Sqs, SqsClient, SendMessageRequest};
use sha2::{Digest, Sha256};
use std::env;

use sqs_lambda::cache::{Cache, CacheResponse, NopCache};
use sqs_lambda::completion_event_serializer::CompletionEventSerializer;
use sqs_lambda::event_decoder::PayloadDecoder;
use sqs_lambda::s3_event_emitter::S3EventEmitter;
use sqs_lambda::event_handler::{Completion, EventHandler, OutputEvent};
use sqs_lambda::event_processor::{EventProcessor, EventProcessorActor};
use sqs_lambda::event_retriever::{S3PayloadRetriever, PayloadRetriever};
use sqs_lambda::redis_cache::RedisCache;
use sqs_lambda::sqs_completion_handler::{CompletionPolicy, SqsCompletionHandler, SqsCompletionHandlerActor};
use sqs_lambda::sqs_consumer::{ConsumePolicy, SqsConsumer, SqsConsumerActor};

use async_trait::async_trait;
use futures::compat::Future01CompatExt;
use sqs_lambda::event_processor::ProcessorState::Complete;
use std::marker::PhantomData;
use chrono::Utc;
use aws_lambda_events::event::s3::{S3Event, S3EventRecord, S3UserIdentity, S3RequestParameters, S3Entity, S3Bucket, S3Object};
use sqs_lambda::local_sqs_service::local_sqs_service;
use tokio::runtime::Runtime;

mod config;

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

#[derive(Debug)]
pub struct AnalyzerDispatcher<S>
    where S: S3 + Send + Sync + 'static
{
    s3_client: Arc<S>
}

impl<S> Clone for AnalyzerDispatcher<S>
    where S: S3 + Send + Sync + 'static
{
    fn clone(&self) -> Self {
        Self {
            s3_client: self.s3_client.clone()
        }
    }
}

async fn get_s3_keys(s3_client: &impl S3, bucket: impl Into<String>) -> Result<impl IntoIterator<Item=Result<String, Error>>, Error>
{
    let bucket = bucket.into();

    let list_res =
        tokio::time::timeout(
            Duration::from_secs(2),
            s3_client.list_objects(
                ListObjectsRequest {
                    bucket,
                    ..Default::default()
                }
            )
        )
        .await??;

    let contents = match list_res.contents {
        Some(contents) => contents,
        None => {
            warn!("List response returned nothing");
            Vec::new()
        }
    };

    Ok(
        contents.into_iter()
            .map(|object| match object.key {
                Some(key) => Ok(key),
                None => bail!("S3Object is missing key")
            })
    )
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


#[async_trait]
impl<S> EventHandler for AnalyzerDispatcher<S>
    where S: S3 + Send + Sync + 'static
{
    type InputEvent = GeneratedSubgraphs;
    type OutputEvent = Vec<AnalyzerDispatchEvent>;
    type Error = sqs_lambda::error::Error<Arc<failure::Error>>;

    async fn handle_event(&mut self, subgraphs: GeneratedSubgraphs) -> OutputEvent<Self::OutputEvent, Self::Error> {
        let bucket = std::env::var("BUCKET_PREFIX")
            .map(|prefix| format!("{}-analyzers-bucket", prefix))
            .expect("BUCKET_PREFIX");

        let mut subgraph = Graph::new(0);

        subgraphs.subgraphs.iter().for_each(|generated_subgraph| subgraph.merge(generated_subgraph));

        if subgraph.is_empty() {
            warn!("Attempted to handle empty subgraph");
            return OutputEvent::new(Completion::Total(vec![]))
        }

        info!("Retrieving S3 keys");
        let keys = match get_s3_keys(self.s3_client.as_ref(), bucket).await {
            Ok(keys) => keys,
            Err(e) => return OutputEvent::new(Completion::Error(
                sqs_lambda::error::Error::ProcessingError(Arc::new(e))
            ))
        };

        let mut dispatch_events = Vec::new();

        let mut failed = None;
        for key in keys {
            let key = match key {
                Ok(key) => key,
                Err(e) => {
                    warn!("Failed to retrieve key with {:?}", e);
                    failed = Some(e);
                    continue
                }
            };

            dispatch_events.push(
              AnalyzerDispatchEvent {
                  key,
                  subgraph: subgraph.clone()
              }
            );
        }

        let completed = if let Some(e) = failed {
            OutputEvent::new(
                Completion::Partial(
                    (
                        dispatch_events,
                        sqs_lambda::error::Error::ProcessingError(Arc::new(e))
                    )
                )
            )
        } else {
            OutputEvent::new(Completion::Total(dispatch_events))
        };

//        identities.into_iter().for_each(|identity| completed.add_identity(identity));

        completed
    }
}

#[derive(Clone, Debug, Default)]
pub struct SubgraphSerializer {
    proto: Vec<u8>,
}

impl CompletionEventSerializer for SubgraphSerializer {
    type CompletedEvent = Vec<AnalyzerDispatchEvent>;
    type Output = Vec<u8>;
    type Error = sqs_lambda::error::Error<Arc<failure::Error>>;

    fn serialize_completed_events(
        &mut self,
        completed_events: &[Self::CompletedEvent],
    ) -> Result<Vec<Self::Output>, Self::Error> {


        let unique_events: Vec<_> = completed_events
            .iter()
            .flatten()
            .collect();

        let mut final_subgraph = Graph::new(0);

        for event in unique_events.iter() {
            final_subgraph.merge(&event.subgraph);
        }

        let mut serialized = Vec::with_capacity(unique_events.len());

        for event in unique_events {
            let event = json!({
                "key": event.key,
                "subgraph": encode_subgraph(&final_subgraph)
                    .map_err(Arc::new)
                    .map_err(|e| {
                        sqs_lambda::error::Error::EncodeError(e.to_string())
                    })?
            });

            serialized.push(
                serde_json::to_vec(&event)
                    .map_err(Arc::new)
                    .map_err(|e| {
                        sqs_lambda::error::Error::EncodeError(e.to_string())
                    })?
            );

        }

        Ok(serialized)
    }
}


#[derive(Debug, Clone, Default)]
pub struct ZstdProtoDecoder;

impl<E> PayloadDecoder<E> for ZstdProtoDecoder
    where E: Message + Default
{
    fn decode(&mut self, body: Vec<u8>) -> Result<E, Box<dyn std::error::Error>>
        where E: Message + Default,
    {
        let mut decompressed = Vec::new();

        let mut body = Cursor::new(&body);

        zstd::stream::copy_decode(&mut body, &mut decompressed)?;

        Ok(E::decode(decompressed)?)
    }
}


fn time_based_key_fn(_event: &[u8]) -> String {
    info!("event length {}", _event.len());
    let cur_ms = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(n) => n.as_millis(),
        Err(_) => panic!("SystemTime before UNIX EPOCH!"),
    };

    let cur_day = cur_ms - (cur_ms % 86400);

    format!(
        "{}/{}-{}",
        cur_day, cur_ms, uuid::Uuid::new_v4()
    )
}

fn map_sqs_message(event: aws_lambda_events::event::sqs::SqsMessage) -> rusoto_sqs::Message {
    rusoto_sqs::Message {
        attributes: Some(event.attributes),
        body: event.body,
        md5_of_body: event.md5_of_body,
        md5_of_message_attributes: event.md5_of_message_attributes,
        message_attributes: None,
        message_id: event.message_id,
        receipt_handle: event.receipt_handle,
    }
}


fn handler(event: SqsEvent, ctx: Context) -> Result<(), HandlerError> {
    info!("Handling event");

    let mut initial_events: HashSet<String> = event.records
        .iter()
        .map(|event| event.message_id.clone().unwrap())
        .collect();

    info!("Initial Events {:?}", initial_events);

    let (tx, rx) = std::sync::mpsc::sync_channel(10);
    let completed_tx = tx.clone();

    std::thread::spawn(move || {
        tokio_compat::run_std(
            async move {
                let queue_url = std::env::var("QUEUE_URL").expect("QUEUE_URL");
                info!("Queue Url: {}", queue_url);
                let bucket_prefix = std::env::var("BUCKET_PREFIX").expect("BUCKET_PREFIX");

                let bucket = bucket_prefix + "-dispatched-analyzer-bucket";
                info!("Output events to: {}", bucket);
                let region = config::region();

                let cache = config::event_cache().await;
                let analyzer_dispatcher
                    = AnalyzerDispatcher {
                        s3_client: Arc::new(S3Client::new(region.clone())),
                    };

                let initial_messages: Vec<_> = event.records
                    .into_iter()
                    .map(map_sqs_message)
                    .collect();

                sqs_lambda::sqs_service::sqs_service(
                    queue_url,
                    initial_messages,
                    bucket,
                    ctx,
                    S3Client::new(region.clone()),
                    SqsClient::new(region.clone()),
                    ZstdProtoDecoder::default(),
                    SubgraphSerializer { proto: Vec::with_capacity(1024) },
                    analyzer_dispatcher,
                    cache.clone(),
                    move |_self_actor, result: Result<String, String>| {
                        match result {
                            Ok(worked) => {
                                info!("Handled an event, which was successfully deleted: {}", &worked);
                                tx.send(worked).unwrap();
                            }
                            Err(worked) => {
                                info!("Handled an initial_event, though we failed to delete it: {}", &worked);
                                tx.send(worked).unwrap();
                            }
                        }
                    },
                    move |_, _| async move {Ok(())},
                ).await;
                completed_tx.clone().send("Completed".to_owned()).unwrap();
            });
    });

    info!("Checking acks");
    for r in &rx {
        info!("Acking event: {}", &r);
        initial_events.remove(&r);
        if r == "Completed" {
            while let Ok(r) = rx.try_recv() {
                initial_events.remove(&r);
            }
            break;
        }
    }

    info!("Completed execution");

    if initial_events.is_empty() {
        info!("Successfully acked all initial events");
        Ok(())
    } else {
        Err(lambda::error::HandlerError::from("Failed to ack all initial events"))
    }
}


fn init_sqs_client() -> SqsClient
{
    info!("Connecting to local us-east-1 http://sqs.us-east-1.amazonaws.com:9324");

    SqsClient::new_with(
        HttpClient::new().expect("failed to create request dispatcher"),
        rusoto_credential::StaticProvider::new_minimal(
            "dummy_sqs".to_owned(),
            "dummy_sqs".to_owned(),
        ),
        Region::Custom {
            name: "us-east-1".to_string(),
            endpoint: "http://sqs.us-east-1.amazonaws.com:9324".to_string(),
        }
    )
}

fn init_s3_client() -> S3Client
{
    info!("Connecting to local http://s3:9000");
    S3Client::new_with(
        HttpClient::new().expect("failed to create request dispatcher"),
        rusoto_credential::StaticProvider::new_minimal(
            "minioadmin".to_owned(),
            "minioadmin".to_owned(),
        ),
        Region::Custom {
            name: "locals3".to_string(),
            endpoint: "http://s3:9000".to_string(),
        },
    )
}

async fn local_handler() -> Result<(), Box<dyn std::error::Error>> {
    std::env::set_var("BUCKET_PREFIX", "local-grapl");
    let analyzer_dispatcher
        = AnalyzerDispatcher {
        s3_client: Arc::new(init_s3_client()),
    };

    local_sqs_service(
        "http://sqs.us-east-1.amazonaws.com:9324/queue/analyzer-dispatcher-queue",
        "local-grapl-analyzer-dispatched-bucket",
        Context {
            deadline: Utc::now().timestamp_millis() + 10_000,
            ..Default::default()
        },
        init_s3_client(),
        init_sqs_client(),
        ZstdProtoDecoder::default(),
        SubgraphSerializer { proto: Vec::with_capacity(1024) },
        analyzer_dispatcher,
        NopCache {},
        |_, event_result | {dbg!(event_result);},
        move |bucket, key| async move {
            let output_event = S3Event {
                records: vec![
                    S3EventRecord {
                        event_version: None,
                        event_source: None,
                        aws_region: None,
                        event_time: chrono::Utc::now(),
                        event_name: None,
                        principal_id: S3UserIdentity { principal_id: None },
                        request_parameters: S3RequestParameters { source_ip_address: None },
                        response_elements: Default::default(),
                        s3: S3Entity {
                            schema_version: None,
                            configuration_id: None,
                            bucket: S3Bucket {
                                name: Some(bucket),
                                owner_identity: S3UserIdentity { principal_id: None },
                                arn: None
                            },
                            object: S3Object {
                                key: Some(key),
                                size: 0,
                                url_decoded_key: None,
                                version_id: None,
                                e_tag: None,
                                sequencer: None
                            }
                        }
                    }
                ]
            };

            let sqs_client = init_sqs_client();

            // publish to SQS
            sqs_client.send_message(
                SendMessageRequest {
                    message_body: serde_json::to_string(&output_event)
                        .expect("failed to encode s3 event"),
                    queue_url: "http://sqs.us-east-1.amazonaws.com:9324/queue/analyzer-executor-queue".to_string(),
                    ..Default::default()
                }
            ).await?;

            Ok(())
        }
    ).await?;

    Ok(())
}


fn main() -> Result<(), Box<dyn std::error::Error>> {
    simple_logger::init_with_level(log::Level::Info).unwrap();

    let is_local = std::env::var("IS_LOCAL")
        .is_ok();

    if is_local {
        info!("Running locally");
        std::thread::sleep_ms(10_000);
        let mut runtime = Runtime::new().unwrap();

        loop {
            if let Err(e) = runtime.block_on(async move { local_handler().await }) {
                error!("{}", e);
            }
            std::thread::sleep_ms(2_000);
        }
    }  else {
        info!("Running in AWS");
        lambda!(handler);
    }


    Ok(())
}

