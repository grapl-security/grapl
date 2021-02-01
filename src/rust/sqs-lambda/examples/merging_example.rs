#![allow(unused_must_use)]

extern crate futures;
extern crate rusoto_s3;
extern crate rusoto_sqs;
extern crate sqs_lambda;
extern crate tokio;

use std::{error::Error,
          fmt::Debug,
          io::{Cursor,
               Stdout}};

use async_trait::async_trait;
use aws_lambda_events::event::s3::{S3Bucket,
                                   S3Entity,
                                   S3Event,
                                   S3EventRecord,
                                   S3Object,
                                   S3RequestParameters,
                                   S3UserIdentity};
use chrono::Utc;
use grapl_observe::metric_reporter::MetricReporter;
use lambda_runtime::Context;
use prost::{bytes::Bytes,
            Message};
use rusoto_core::Region;
use rusoto_s3::S3Client;
use rusoto_sqs::SqsClient;
use serde::Deserialize;
use sqs_lambda::{cache::{Cache,
                         NopCache},
                 completion_event_serializer::CompletionEventSerializer,
                 error::Error as SqsLambdaError,
                 event_decoder::PayloadDecoder,
                 event_handler::{Completion,
                                 EventHandler,
                                 OutputEvent},
                 local_sqs_service::local_sqs_service};
use tracing_subscriber::EnvFilter;

struct MyService<C>
where
    C: Cache + Clone + Send + Sync + 'static,
{
    cache: C,
}

impl<C> Clone for MyService<C>
where
    C: Cache + Clone + Send + Sync + 'static,
{
    fn clone(&self) -> MyService<C> {
        Self {
            cache: self.cache.clone(),
        }
    }
}

impl<C> MyService<C>
where
    C: Cache + Clone + Send + Sync + 'static,
{
    pub fn new(cache: C) -> Self {
        Self { cache }
    }
}

#[async_trait]
impl<C> EventHandler for MyService<C>
where
    C: Cache + Clone + Send + Sync + 'static,
{
    type InputEvent = Vec<u8>;
    type OutputEvent = Subgraph;
    type Error = SqsLambdaError;

    async fn handle_event(
        &mut self,
        _input: Self::InputEvent,
    ) -> OutputEvent<Self::OutputEvent, Self::Error> {
        // do some work
        let completed = OutputEvent::new(Completion::Total(Subgraph {}));

        // for input in _input.keys() {
        //     completed.add_identity(input);
        // }

        completed
    }
}

#[derive(Clone, Debug)]
pub struct Subgraph {}

impl Subgraph {
    #[allow(dead_code)] // Ultimately need an implementation for this example
    fn merge(&mut self, _other: &Self) {
        unimplemented!()
    }

    #[allow(dead_code)] // Ultimately need an implementation for this example
    fn into_bytes(self) -> Vec<u8> {
        unimplemented!()
    }
}

#[derive(Clone, Debug)]
pub struct SubgraphSerializer {}

impl CompletionEventSerializer for SubgraphSerializer {
    type CompletedEvent = Subgraph;
    type Output = Vec<u8>;
    type Error = SqsLambdaError;

    fn serialize_completed_events(
        &mut self,
        completed_events: &[Self::CompletedEvent],
    ) -> Result<Vec<Self::Output>, Self::Error> {
        let mut _subgraph = Subgraph {};
        for _sg in completed_events {
            // subgraph.merge(sg);
        }

        // subgraph.into_bytes()
        Ok(vec![])
    }
}

#[derive(Clone)]
pub struct ZstdProtoDecoder;

impl<E> PayloadDecoder<E> for ZstdProtoDecoder
where
    E: Message + Default,
{
    fn decode(&mut self, body: Vec<u8>) -> Result<E, Box<dyn Error>>
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

#[derive(Clone, Default)]
pub struct ZstdDecoder {
    pub buffer: Vec<u8>,
}

impl PayloadDecoder<Vec<u8>> for ZstdDecoder {
    fn decode(&mut self, body: Vec<u8>) -> Result<Vec<u8>, Box<dyn Error>> {
        self.buffer.clear();

        let mut body = Cursor::new(&body);

        zstd::stream::copy_decode(&mut body, &mut self.buffer)?;

        Ok(self.buffer.clone())
    }
}

#[derive(Clone, Default)]
pub struct ZstdJsonDecoder {
    pub buffer: Vec<u8>,
}

impl<E> PayloadDecoder<E> for ZstdJsonDecoder
where
    E: for<'a> Deserialize<'a>,
{
    fn decode(&mut self, body: Vec<u8>) -> Result<E, Box<dyn Error>> {
        self.buffer.clear();

        let mut body = Cursor::new(&body);

        zstd::stream::copy_decode(&mut body, &mut self.buffer)?;

        Ok(serde_json::from_slice(&self.buffer[..])?)
    }
}

fn init_sqs_client() -> SqsClient {
    SqsClient::new(Region::Custom {
        name: "localsqs".to_string(),
        endpoint: "http://localhost:9324".to_string(),
    })
}

fn init_s3_client() -> S3Client {
    S3Client::new(Region::Custom {
        name: "locals3".to_string(),
        endpoint: "http://localhost:4572".to_string(),
    })
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let filter = EnvFilter::from_default_env();
    tracing_subscriber::fmt()
        // .json()
        // .with_max_level(Level::DEBUG)
        .with_env_filter(filter)
        .init();

    // simple_logger::init_with_level(Level::Info).unwrap();
    // let cache = RedisCache::new("address".to_owned()).await.expect("Could not create redis client");
    tracing::info!("Initializing service");
    let service = MyService::new(NopCache {});

    local_sqs_service(
        "http://localhost:9324/queue/sysmon-graph-generator-queue",
        "unid-subgraphs-generated",
        Context {
            deadline: Utc::now().timestamp_millis() + 300_000,
            ..Default::default()
        },
        |_| init_s3_client(),
        init_s3_client(),
        init_sqs_client(),
        ZstdJsonDecoder { buffer: vec![] },
        SubgraphSerializer {},
        service,
        NopCache {},
        MetricReporter::<Stdout>::new("test-service"),
        |_, event_result| {
            dbg!(event_result);
        },
        move |bucket, key| async move {
            let _output_event = S3Event {
                records: vec![S3EventRecord {
                    event_version: None,
                    event_source: None,
                    aws_region: None,
                    event_time: chrono::Utc::now(),
                    event_name: None,
                    principal_id: S3UserIdentity { principal_id: None },
                    request_parameters: S3RequestParameters {
                        source_ip_address: None,
                    },
                    response_elements: Default::default(),
                    s3: S3Entity {
                        schema_version: None,
                        configuration_id: None,
                        bucket: S3Bucket {
                            name: Some(bucket),
                            owner_identity: S3UserIdentity { principal_id: None },
                            arn: None,
                        },
                        object: S3Object {
                            key: Some(key),
                            size: None,
                            url_decoded_key: None,
                            version_id: None,
                            e_tag: None,
                            sequencer: None,
                        },
                    },
                }],
            };

            let _sqs_client = init_sqs_client();

            // publish to SQS
            // sqs_client.send_message(
            //     SendMessageRequest {
            //         message_body: serde_json::to_string(&output_event)
            //             .expect("failed to encode s3 event"),
            //         queue_url: "http://localhost:9324/queue/node-identifier-retry-queue".to_string(),
            //         ..Default::default()
            //     }
            // ).await;

            Ok(())
        },
    )
    .await;

    Ok(())
}
