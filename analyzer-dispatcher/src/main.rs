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
use rusoto_core::Region;
use rusoto_s3::{ListObjectsRequest, PutObjectRequest, S3, S3Client};
use rusoto_sns::{Sns, SnsClient};
use rusoto_sns::PublishInput;
use rusoto_sqs::{GetQueueUrlRequest, Sqs, SqsClient};
use sha2::{Digest, Sha256};
use std::env;

use sqs_lambda::cache::{Cache, CacheResponse};
use sqs_lambda::completion_event_serializer::CompletionEventSerializer;
use sqs_lambda::event_decoder::PayloadDecoder;
use sqs_lambda::event_emitter::S3EventEmitter;
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

    let list_res = s3_client.list_objects(
        ListObjectsRequest {
            bucket,
            ..Default::default()
        }
    )
        .with_timeout(Duration::from_secs(2))
        .compat()
        .await?;

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
    type Error = Arc<failure::Error>;

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
            Err(e) => return OutputEvent::new(Completion::Error(Arc::new(e)))
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
                    (dispatch_events, Arc::new(e))
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
    type Error = failure::Error;

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
                "subgraph": encode_subgraph(&final_subgraph)?
            });

            serialized.push(
                serde_json::to_vec(&event)?
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

    std::thread::spawn(move || {
        tokio_compat::run_std(
            async move {
                let queue_url = std::env::var("QUEUE_URL").expect("QUEUE_URL");
                info!("Queue Url: {}", queue_url);

                let bucket = std::env::var("DISPATCHED_ANALYZER_BUCKET")
                    .expect("DISPATCHED_ANALYZER_TOPIC_ARN");

                info!("Output events to: {}", bucket);
                let region = {
                    let region_str = std::env::var("AWS_REGION").expect("AWS_REGION");
                    Region::from_str(&region_str).expect("Region error")
                };

                let cache_address = {
                    let dispatch_event_cache_addr = std::env::var("DISPATCH_EVENT_CACHE_ADDR").expect("DISPATCH_EVENT_CACHE_ADDR");
                    let dispatch_event_cache_port = std::env::var("DISPATCH_EVENT_CACHE_PORT").expect("DISPATCH_EVENT_CACHE_PORT");

                    format!(
                        "{}:{}",
                        dispatch_event_cache_addr,
                        dispatch_event_cache_port,
                    )
                };
                let cache = RedisCache::new(cache_address.to_owned()).await.expect("Could not create redis client");

                let node_identifier = AnalyzerDispatcher {
                    s3_client: Arc::new(S3Client::new(region.clone())),
                };


                info!("SqsCompletionHandler");

                let finished_tx = tx.clone();
                let sqs_completion_handler = SqsCompletionHandlerActor::new(
                    SqsCompletionHandler::new(
                        SqsClient::new(region.clone()),
                        queue_url.to_string(),
                        SubgraphSerializer { proto: Vec::with_capacity(1024) },
                        S3EventEmitter::new(
                            S3Client::new(region.clone()),
                            bucket.to_owned(),
                            time_based_key_fn,
                        ),
                        CompletionPolicy::new(
                            1000, // Buffer up to 1000 messages
                            Duration::from_secs(30), // Buffer for up to 30 seconds
                        ),
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
                        cache.clone()
                    )
                );


                info!("Defining consume policy");
                let consume_policy = ConsumePolicy::new(
                    ctx, // Use the Context.deadline from the lambda_runtime
                    Duration::from_secs(10), // Stop consuming when there's 2 seconds left in the runtime
                    3, // If we get 3 empty receives in a row, stop consuming
                );

                info!("Defining consume policy");
                let (shutdown_tx, shutdown_notify) = tokio::sync::oneshot::channel();

                info!("SqsConsumer");
                let sqs_consumer = SqsConsumerActor::new(
                    SqsConsumer::new(
                        SqsClient::new(region.clone()),
                        queue_url.clone(),
                        consume_policy,
                        sqs_completion_handler.clone(),
                        shutdown_tx,
                    )
                );

                info!("EventProcessors");
                let event_processors: Vec<_> = (0..10)
                    .map(|_| {
                        EventProcessorActor::new(EventProcessor::new(
                            sqs_consumer.clone(),
                            sqs_completion_handler.clone(),
                            node_identifier.clone(),
                            S3PayloadRetriever::new(S3Client::new(region.clone()), ZstdProtoDecoder::default())
                        ))
                    })
                    .collect();

                info!("Start Processing");

                futures::future::join_all(event_processors.iter().map(|ep| ep.start_processing())).await;

                let mut proc_iter = event_processors.iter().cycle();
                for event in event.records {
                    let next_proc = proc_iter.next().unwrap();
                    next_proc.process_event(
                        map_sqs_message(event)
                    ).await;
                }

                info!("Waiting for shutdown notification");

                // Wait for the consumers to shutdown
                let _ = shutdown_notify.await;
                info!("Consumer shutdown");
                finished_tx.send("Completed".to_owned()).unwrap();
            });
    });

    info!("Checking acks");
    for r in &rx {
        info!("Acking event: {}", &r);
        initial_events.remove(&r);
        if r == "Completed" {
            let r = rx.recv_timeout(Duration::from_millis(100));
            if let Ok(r) = r {
                initial_events.remove(&r);
            }
            // If we're done go ahead and try to clear out any remaining
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



fn main() {
    simple_logger::init_with_level(log::Level::Info).unwrap();

    lambda!(handler);
}

