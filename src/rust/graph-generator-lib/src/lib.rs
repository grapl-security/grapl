use std::collections::HashSet;
use std::io::Cursor;
use std::sync::Arc;
use std::time::Duration;

use std::fmt::Debug;

use log::*;
use rusoto_core::{HttpClient, Region};
use rusoto_s3::S3Client;
use rusoto_sqs::{SendMessageRequest, Sqs, SqsClient};
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

#[derive(Clone, Default)]
pub struct ZstdJsonDecoder {
    pub buffer: Vec<u8>,
}

impl<D> PayloadDecoder<D> for ZstdJsonDecoder
where
    for<'a> D: Deserialize<'a>,
{
    fn decode(&mut self, body: Vec<u8>) -> Result<D, Box<dyn std::error::Error>> {
        self.buffer.clear();

        let mut body = Cursor::new(&body);

        zstd::stream::copy_decode(&mut body, &mut self.buffer)?;

        Ok(serde_json::from_slice(&self.buffer)?)
    }
}

#[derive(Clone, Debug, Default)]
pub struct SubgraphSerializer {
    proto: Vec<u8>,
}

impl CompletionEventSerializer for SubgraphSerializer {
    type CompletedEvent = Graph;
    type Output = Vec<u8>;
    type Error = sqs_lambda::error::Error;

    fn serialize_completed_events(
        &mut self,
        completed_events: &[Self::CompletedEvent],
    ) -> Result<Vec<Self::Output>, Self::Error> {
        let mut subgraph = Graph::new(0);

        let mut pre_nodes = 0;
        let mut pre_edges = 0;

        for sg in completed_events.iter() {
            pre_nodes += sg.nodes.len();
            pre_edges += sg.edges.len();
            subgraph.merge(sg);
        }

        if subgraph.is_empty() {
            warn!(
                concat!(
                    "Output subgraph is empty. Serializing to empty vector.",
                    "pre_nodes: {} pre_edges: {}"
                ),
                pre_nodes, pre_edges,
            );
            return Ok(vec![]);
        }

        info!(
            "Serializing {} nodes {} edges. Down from {} nodes {} edges.",
            subgraph.nodes.len(),
            subgraph.edges.len(),
            pre_nodes,
            pre_edges,
        );

        let subgraphs = GeneratedSubgraphs {
            subgraphs: vec![subgraph],
        };

        self.proto.clear();

        prost::Message::encode(&subgraphs, &mut self.proto)
            .map_err(|e| sqs_lambda::error::Error::EncodeError(e.to_string()))?;

        let mut compressed = Vec::with_capacity(self.proto.len());
        let mut proto = Cursor::new(&self.proto);
        zstd::stream::copy_encode(&mut proto, &mut compressed, 4)
            .map_err(|e| sqs_lambda::error::Error::EncodeError(e.to_string()))?;

        Ok(vec![compressed])
    }
}

pub fn map_sqs_message(event: aws_lambda_events::event::sqs::SqsMessage) -> rusoto_sqs::Message {
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

pub async fn local_service<
    IE: Send + Sync + Clone + 'static,
    EH: EventHandler<InputEvent = IE, OutputEvent = Graph, Error = sqs_lambda::error::Error>
        + Send
        + Sync
        + Clone
        + 'static,
    ED: PayloadDecoder<IE> + Send + Sync + Clone + 'static,
>(
    queue_url: String,
    generator: EH,
    event_decoder: ED,
) -> Result<(), Box<dyn std::error::Error>> {
    let queue_name = queue_url.split("/").last().unwrap();
    grapl_config::wait_for_sqs(init_sqs_client(), queue_name).await?;
    grapl_config::wait_for_sqs(init_sqs_client(), "node-identifier-queue").await?;
    grapl_config::wait_for_s3(init_s3_client()).await?;

    local_sqs_service(
        queue_url,
        "local-grapl-unid-subgraphs-generated-bucket",
        Context {
            deadline: Utc::now().timestamp_millis() + 10_000,
            ..Default::default()
        },
        |_| init_s3_client(),
        init_s3_client(),
        init_sqs_client(),
        event_decoder,
        SubgraphSerializer {
            proto: Vec::with_capacity(1024),
        },
        generator,
        NopCache {},
        |_, event_result| {
            debug!("{:?}", event_result);
        },
        move |bucket, key| async move {
            let output_event = S3Event {
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
                            size: Some(0),
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
                    queue_url:
                        "http://sqs.us-east-1.amazonaws.com:9324/queue/node-identifier-queue"
                            .to_string(),
                    ..Default::default()
                })
                .await?;

            Ok(())
        },
    )
    .await?;
    Ok(())
}

pub fn run_graph_generator_aws<
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
    lambda_runtime::lambda!(move |event, context| {
        handler(event, context, generator.clone(), event_decoder.clone())
    })
}

fn handler<
    IE: Send + Sync + Clone + 'static,
    EH: EventHandler<InputEvent = IE, OutputEvent = Graph, Error = sqs_lambda::error::Error>
        + Send
        + Sync
        + Clone
        + 'static,
    ED: PayloadDecoder<IE> + Send + Sync + Clone + 'static,
>(
    event: SqsEvent,
    ctx: Context,
    generator: EH,
    event_decoder: ED,
) -> Result<(), HandlerError> {
    info!("Handling event");

    let mut initial_events: HashSet<String> = event
        .records
        .iter()
        .map(|event| event.message_id.clone().unwrap())
        .collect();

    info!("Initial Events {:?}", initial_events);

    let (tx, rx) = std::sync::mpsc::sync_channel(10);
    let completed_tx = tx.clone();

    let generator = generator.clone();
    let event_decoder = event_decoder.clone();
    std::thread::spawn(move || {
        let generator = generator.clone();
        let event_decoder = event_decoder.clone();
        tokio_compat::run_std(async move {
            let queue_url = std::env::var("QUEUE_URL").expect("QUEUE_URL");
            info!("Queue Url: {}", queue_url);
            let bucket_prefix = std::env::var("BUCKET_PREFIX").expect("BUCKET_PREFIX");

            let bucket = bucket_prefix + "-unid-subgraphs-generated-bucket";
            info!("Output events to: {}", bucket);
            let region = config::region();

            let cache = config::event_cache().await;

            let initial_messages: Vec<_> = event.records.into_iter().map(map_sqs_message).collect();

            sqs_lambda::sqs_service::sqs_service(
                queue_url,
                initial_messages,
                bucket,
                ctx,
                |region_str| S3Client::new(Region::from_str(&region_str).expect("region_str")),
                S3Client::new(region.clone()),
                SqsClient::new(region.clone()),
                event_decoder.clone(),
                SubgraphSerializer {
                    proto: Vec::with_capacity(1024),
                },
                generator,
                cache.clone(),
                move |_self_actor, result: Result<String, String>| match result {
                    Ok(worked) => {
                        info!(
                            "Handled an event, which was successfully deleted: {}",
                            &worked
                        );
                        tx.send(worked).unwrap();
                    }
                    Err(worked) => {
                        info!(
                            "Handled an event, though we failed to delete it: {}",
                            &worked
                        );
                        tx.send(worked).unwrap();
                    }
                },
                move |bucket, key| async move {
                    info!("Emitted event to {} {}", bucket, key);
                    Ok(())
                },
            )
            .await
            .expect("service failed");
            completed_tx.clone().send("Completed".to_owned()).unwrap();
        });
    });

    info!("Checking acks");
    for r in &rx {
        info!("Acking event: {}", &r);
        initial_events.remove(&r);
        if r == "Completed" {
            // If we're done go ahead and try to clear out any remaining
            while let Ok(r) = rx.recv_timeout(Duration::from_millis(100)) {
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

pub async fn run_graph_generator_local<
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
    let queue_url = std::env::var("QUEUE_URL").expect("QUEUE_URL");

    loop {
        let queue_url = queue_url.clone();
        let generator = generator.clone();
        let event_decoder = event_decoder.clone();

        if let Err(e) =
            local_service(queue_url.clone(), generator.clone(), event_decoder.clone()).await
        {
            error!("{}", e);
            std::thread::sleep(Duration::from_secs(2));
        }
    }
}

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
    let is_local = std::env::var("IS_LOCAL");

    info!("IS_LOCAL={:?}", is_local);
    if is_local
        .map(|is_local| is_local.to_lowercase().parse().unwrap_or(false))
        .unwrap_or(false)
    {
        run_graph_generator_local(generator, event_decoder).await;
    } else {
        run_graph_generator_aws(generator, event_decoder);
    }
}
