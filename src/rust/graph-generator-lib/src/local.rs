use crate::serialization::SubgraphSerializer;
use aws_lambda_events::event::s3::{
    S3Bucket, S3Entity, S3Event, S3EventRecord, S3Object, S3RequestParameters, S3UserIdentity,
};
use chrono::Utc;
use graph_descriptions::graph_description::*;
use lambda_runtime::Context;
use log::*;
use rusoto_core::{HttpClient, Region};
use rusoto_s3::S3Client;
use rusoto_sqs::{SendMessageRequest, Sqs, SqsClient};
use sqs_lambda::cache::NopCache;
use sqs_lambda::event_decoder::PayloadDecoder;
use sqs_lambda::event_handler::EventHandler;
use sqs_lambda::local_sqs_service::local_sqs_service;
use std::time::Duration;

/// Performs some initial steps before starting a local sqs-based service with the provided [EventHandler]
async fn initialize_local_service<
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
    grapl_config::wait_for_sqs(init_local_sqs_client(), queue_name).await?;
    grapl_config::wait_for_sqs(init_local_sqs_client(), "grapl-node-identifier-queue").await?;
    grapl_config::wait_for_s3(init_local_s3_client()).await?;

    let destination_bucket = "local-grapl-unid-subgraphs-generated-bucket";
    let local_s3_client = init_local_s3_client();
    let local_sqs_client = init_local_sqs_client();

    let subgraph_serializer = SubgraphSerializer::new(Vec::with_capacity(1024));

    /*
     * queue_url - The queue to be reading incoming log events from.
     * destination_bucket - The destination S3 bucket where completed, serialized subgraphs should be written to.
     * local_s3_client - A local s3 client
     * local_sqs_client - A local sqs client
     * subgraph_serializer - An event encoder used to serialize the subgraphs created by the provided generator.
     * generator - An EventHandler that takes in log events, parses them, and generates subgraphs based on the logs provided.
     */
    local_sqs_service(
        queue_url,
        destination_bucket,
        Context {
            deadline: Utc::now().timestamp_millis() + 10_000,
            ..Default::default()
        },
        |_| init_local_s3_client(),
        local_s3_client,
        local_sqs_client,
        event_decoder,
        subgraph_serializer,
        generator,
        NopCache {},
        |_, event_result| {
            debug!("{:?}", event_result);
        },
        move |bucket, key| local_emit_event(bucket, key),
    )
    .await?;
    Ok(())
}

async fn local_emit_event(
    bucket: String,
    key: String,
) -> Result<(), Box<(dyn std::error::Error + Send + Sync + 'static)>> {
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
                    size: Some(0),
                    url_decoded_key: None,
                    version_id: None,
                    e_tag: None,
                    sequencer: None,
                },
            },
        }],
    };

    let sqs_client = init_local_sqs_client();

    // publish to SQS
    sqs_client
        .send_message(SendMessageRequest {
            message_body: serde_json::to_string(&output_event).expect("failed to encode s3 event"),
            queue_url: "http://sqs.us-east-1.amazonaws.com:9324/queue/grapl-node-identifier-queue"
                .to_string(),
            ..Default::default()
        })
        .await?;

    Ok(())
}

/// Runs the provided graph generator implementation locally.
///
/// Expects SOURCE_QUEUE_URL environment variable is set
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
    let source_queue_url = std::env::var("SOURCE_QUEUE_URL").expect("SOURCE_QUEUE_URL");

    loop {
        let queue_url = source_queue_url.clone();
        let generator = generator.clone();
        let event_decoder = event_decoder.clone();

        if let Err(e) = initialize_local_service(queue_url, generator, event_decoder).await {
            error!("{}", e);
            std::thread::sleep(Duration::from_secs(2));
        }
    }
}

/// Creates an SQS client for local testing.
///
/// Note: Region endpoint address is set in docker-compose for the container to link to a local sqs service.
/// The endpoint does NOT point to real SQS.
fn init_local_sqs_client() -> SqsClient {
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

/// Creates an S3 Client used when running locally.
fn init_local_s3_client() -> S3Client {
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
