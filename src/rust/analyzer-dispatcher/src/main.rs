#![type_length_limit = "1214269"]
// Our types are simply too powerful

use grapl_observe::metric_reporter::MetricReporter;
use std::collections::HashSet;
use std::io::{Cursor, Stdout};
use std::sync::Arc;
use std::time::Duration;

use aws_lambda_events::event::sqs::SqsEvent;
use bytes::Bytes;
use failure::{bail, Error};
use grapl_graph_descriptions::graph_description::*;
use lambda_runtime::error::HandlerError;
use lambda_runtime::lambda;
use lambda_runtime::Context;
use log::{debug, error, info, warn};
use prost::Message;
use rusoto_core::{HttpClient, Region};
use rusoto_s3::{ListObjectsRequest, S3Client, S3};
use rusoto_sqs::{SendMessageRequest, Sqs, SqsClient};
use serde::{Deserialize, Serialize};
use serde_json::json;

use sqs_lambda::cache::NopCache;
use sqs_lambda::completion_event_serializer::CompletionEventSerializer;
use sqs_lambda::event_decoder::PayloadDecoder;
use sqs_lambda::event_handler::{Completion, EventHandler, OutputEvent};

use async_trait::async_trait;
use aws_lambda_events::event::s3::{
    S3Bucket, S3Entity, S3Event, S3EventRecord, S3Object, S3RequestParameters, S3UserIdentity,
};
use chrono::Utc;
use sqs_lambda::local_sqs_service::local_sqs_service_with_options;
use sqs_lambda::local_sqs_service_options::LocalSqsServiceOptionsBuilder;
use sqs_lambda::sqs_completion_handler::CompletionPolicy;
use sqs_lambda::sqs_consumer::ConsumePolicyBuilder;
use std::str::FromStr;

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

#[async_trait]
impl<S> EventHandler for AnalyzerDispatcher<S>
where
    S: S3 + Send + Sync + 'static,
{
    type InputEvent = GeneratedSubgraphs;
    type OutputEvent = Vec<AnalyzerDispatchEvent>;
    type Error = sqs_lambda::error::Error;

    async fn handle_event(
        &mut self,
        subgraphs: GeneratedSubgraphs,
    ) -> OutputEvent<Self::OutputEvent, Self::Error> {
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
            return OutputEvent::new(Completion::Total(vec![]));
        }

        info!("Retrieving S3 keys");
        let keys = match get_s3_keys(self.s3_client.as_ref(), &bucket).await {
            Ok(keys) => keys,
            Err(e) => {
                return OutputEvent::new(Completion::Error(
                    sqs_lambda::error::Error::ProcessingError(format!(
                        "Failed to list bucket: {} with {:?}",
                        bucket, e
                    )),
                ))
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

        let completed = if let Some(e) = failed {
            OutputEvent::new(Completion::Partial((
                dispatch_events,
                sqs_lambda::error::Error::ProcessingError(e.to_string()),
            )))
        } else {
            OutputEvent::new(Completion::Total(dispatch_events))
        };

        // identities.into_iter().for_each(|identity| completed.add_identity(identity));

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
    type Error = sqs_lambda::error::Error;

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
                "subgraph": encode_subgraph(&final_subgraph)
                    .map_err(|e| {
                        sqs_lambda::error::Error::EncodeError(e.to_string())
                    })?
            });

            serialized.push(
                serde_json::to_vec(&event)
                    .map_err(|e| sqs_lambda::error::Error::EncodeError(e.to_string()))?,
            );
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

    let mut initial_events: HashSet<String> = event
        .records
        .iter()
        .map(|event| event.message_id.clone().unwrap())
        .collect();

    info!("Initial Events {:?}", initial_events);

    let (tx, rx) = std::sync::mpsc::sync_channel(10);
    let completed_tx = tx.clone();

    std::thread::spawn(move || {
        tokio_compat::run_std(async move {
            let source_queue_url = std::env::var("SOURCE_QUEUE_URL").expect("SOURCE_QUEUE_URL");
            debug!("Queue Url: {}", source_queue_url);
            let bucket_prefix = std::env::var("BUCKET_PREFIX").expect("BUCKET_PREFIX");

            let bucket = bucket_prefix + "-dispatched-analyzer-bucket";
            info!("Output events to: {}", bucket);
            let region = grapl_config::region();

            let cache = grapl_config::event_cache().await;
            let analyzer_dispatcher = AnalyzerDispatcher {
                s3_client: Arc::new(S3Client::new(region.clone())),
            };

            let initial_messages: Vec<_> = event.records.into_iter().map(map_sqs_message).collect();
            let completion_policy = ConsumePolicyBuilder::default()
                .with_max_empty_receives(1)
                .with_stop_at(Duration::from_secs(10));

            sqs_lambda::sqs_service::sqs_service(
                source_queue_url,
                initial_messages,
                bucket,
                completion_policy.build(ctx),
                CompletionPolicy::new(10, Duration::from_secs(2)),
                |region_str| S3Client::new(Region::from_str(&region_str).expect("region_str")),
                S3Client::new(region.clone()),
                SqsClient::new(region.clone()),
                ZstdProtoDecoder::default(),
                SubgraphSerializer {
                    proto: Vec::with_capacity(1024),
                },
                analyzer_dispatcher,
                cache.clone(),
                MetricReporter::<Stdout>::new("analyzer-dispatcher"),
                move |_self_actor, result: Result<String, String>| match result {
                    Ok(worked) => {
                        info!(
                            "Handled an event, which was successfully deleted: {}",
                            &worked
                        );
                        tx.send(worked).unwrap();
                    }
                    Err(worked) => {
                        warn!(
                            "Handled an initial_event, though we failed to delete it: {}",
                            &worked
                        );
                        tx.send(worked).unwrap();
                    }
                },
                move |_, _| async move { Ok(()) },
            )
            .await;
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
        Err(lambda_runtime::error::HandlerError::from(
            "Failed to ack all initial events",
        ))
    }
}

fn init_sqs_client() -> SqsClient {
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
        },
    )
}

fn init_s3_client() -> S3Client {
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
    let analyzer_dispatcher = AnalyzerDispatcher {
        s3_client: Arc::new(init_s3_client()),
    };

    let source_queue_url = std::env::var("SOURCE_QUEUE_URL").expect("SOURCE_QUEUE_URL");
    let mut options_builder = LocalSqsServiceOptionsBuilder::default();
    options_builder.with_minimal_buffer_completion_policy();

    local_sqs_service_with_options(
        source_queue_url,
        "local-grapl-analyzer-dispatched-bucket",
        Context {
            deadline: Utc::now().timestamp_millis() + 10_000,
            ..Default::default()
        },
        |_| init_s3_client(),
        init_s3_client(),
        init_sqs_client(),
        ZstdProtoDecoder::default(),
        SubgraphSerializer {
            proto: Vec::with_capacity(1024),
        },
        analyzer_dispatcher,
        NopCache {},
        MetricReporter::<Stdout>::new("analyzer-dispatcher"),
        |_, event_result| {
            dbg!(event_result);
        },
        move |bucket, key| async move {
            let output_event = S3Event {
                records: vec![S3EventRecord {
                    event_version: None,
                    event_source: None,
                    aws_region: Some("us-east-1".to_owned()),
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
                            size: 0,
                            url_decoded_key: None,
                            version_id: None,
                            e_tag: None,
                            sequencer: None,
                        },
                    },
                }],
            };

            let sqs_client = init_sqs_client();

            // publish to SQS
            sqs_client
                .send_message(SendMessageRequest {
                    message_body: serde_json::to_string(&output_event)
                        .expect("failed to encode s3 event"),
                    queue_url: std::env::var("ANALYZER_EXECUTOR_QUEUE_URL")
                        .expect("ANALYZER_EXECUTOR_QUEUE_URL"),
                    ..Default::default()
                })
                .await?;

            Ok(())
        },
        options_builder.build(),
    )
    .await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let env = grapl_config::init_grapl_env!();

    if env.is_local {
        info!("Running locally");
        let source_queue_url = std::env::var("SOURCE_QUEUE_URL").expect("SOURCE_QUEUE_URL");

        grapl_config::wait_for_sqs(
            init_sqs_client(),
            source_queue_url.split("/").last().unwrap(),
        )
        .await?;
        grapl_config::wait_for_s3(init_s3_client()).await?;

        loop {
            if let Err(e) = local_handler().await {
                error!("local_handler: {}", e);
            };
        }
    } else {
        info!("Running in AWS");
        lambda!(handler);
    }

    Ok(())
}
