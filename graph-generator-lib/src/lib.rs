use std::io::Cursor;
use std::str::FromStr;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use std::time::UNIX_EPOCH;

use aws_lambda_events::event::s3::{S3Bucket, S3Entity, S3Event, S3EventRecord, S3Object, S3RequestParameters, S3UserIdentity};
use chrono::Utc;
use failure::Error;
use futures::future::Future;
use graph_descriptions::graph_description::*;
use lambda_runtime::Context;
use log::*;
use prost::Message;
use rusoto_core::{HttpClient, Region};
use rusoto_s3::{PutObjectRequest, S3, S3Client};
use rusoto_sqs::{SendMessageRequest, Sqs, SqsClient};
use serde_json::Value;
use sha2::{Digest, Sha256};
use sqs_lambda::cache::NopCache;
use sqs_lambda::completion_event_serializer::CompletionEventSerializer;
use sqs_lambda::event_decoder::PayloadDecoder;
use sqs_lambda::event_handler::EventHandler;
use sqs_lambda::local_sqs_service::local_sqs_service;
use tokio::runtime::Runtime;

#[derive(Debug, Clone, Default)]
pub struct ZstdDecoder;

impl PayloadDecoder<Vec<u8>> for ZstdDecoder
{
    fn decode(&mut self, body: Vec<u8>) -> Result<Vec<u8>, Box<dyn std::error::Error>>
    {
        let mut decompressed = Vec::new();

        let mut body = Cursor::new(&body);

        zstd::stream::copy_decode(&mut body, &mut decompressed)?;

        Ok(decompressed)
    }
}


#[derive(Clone, Debug, Default)]
pub struct SubgraphSerializer {
    proto: Vec<u8>,
}

impl CompletionEventSerializer for SubgraphSerializer {
    type CompletedEvent = Graph;
    type Output = Vec<u8>;
    type Error = sqs_lambda::error::Error<Arc<failure::Error>>;

    fn serialize_completed_events(
        &mut self,
        completed_events: &[Self::CompletedEvent],
    ) -> Result<Vec<Self::Output>, Self::Error> {
        let mut subgraph = Graph::new(
            0
        );

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
                pre_nodes,
                pre_edges,
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

        let subgraphs = GeneratedSubgraphs { subgraphs: vec![subgraph] };

        self.proto.clear();

        prost::Message::encode(&subgraphs, &mut self.proto)
            .map_err(Arc::new)
            .map_err(|e| {
                sqs_lambda::error::Error::EncodeError(e.to_string())
            })?;

        let mut compressed = Vec::with_capacity(self.proto.len());
        let mut proto = Cursor::new(&self.proto);
        zstd::stream::copy_encode(&mut proto, &mut compressed, 4)
            .map_err(Arc::new)
            .map_err(|e| {
                sqs_lambda::error::Error::EncodeError(e.to_string())
            })?;

        Ok(vec![compressed])
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
        },
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

pub async fn local_service<
    EH: EventHandler<
        InputEvent=Vec<u8>,
        OutputEvent=Graph,
        Error=sqs_lambda::error::Error<Arc<failure::Error>>
    > + Send + Sync + Clone + 'static
>(
    bucket_name: String,
    queue_url: String,
    generator: EH,
) -> Result<(), Box<dyn std::error::Error>> {
    local_sqs_service(
        queue_url,
        bucket_name,
        Context {
            deadline: Utc::now().timestamp_millis() + 10_000,
            ..Default::default()
        },
        init_s3_client(),
        init_sqs_client(),
        ZstdDecoder::default(),
        SubgraphSerializer { proto: Vec::with_capacity(1024) },
        generator,
        NopCache {},
        |_, event_result| { dbg!(event_result); },
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
                    }
                ]
            };

            let sqs_client = init_sqs_client();

            // publish to SQS
            sqs_client.send_message(
                SendMessageRequest {
                    message_body: serde_json::to_string(&output_event)
                        .expect("failed to encode s3 event"),
                    queue_url: "http://sqs.us-east-1.amazonaws.com:9324/queue/node-identifier-queue".to_string(),
                    ..Default::default()
                }
            ).await?;

            Ok(())
        },
    ).await?;
    Ok(())
}


pub fn run_graph_generator_local<
    EH: EventHandler<
        InputEvent=Vec<u8>,
        OutputEvent=Graph,
        Error=sqs_lambda::error::Error<Arc<failure::Error>>
    > + Send + Sync + Clone + 'static
>
(
    bucket_name: String,
    queue_url: String,
    generator: EH,
) {
    std::thread::sleep_ms(10_000);
    let mut runtime = Runtime::new().unwrap();

    loop {
        let bucket_name = bucket_name.clone();
        let queue_url = queue_url.clone();
        let generator = generator.clone();

        if let Err(e) = runtime.block_on(async move {
            local_service(
                bucket_name.clone(),
                queue_url.clone(),
                generator.clone(),
            ).await
        }) {
            error!("{}", e);
            std::thread::sleep_ms(2_000);
        }
    }
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}





