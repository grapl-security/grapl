use crate::serialization::SubgraphSerializer;
use aws_lambda_events::event::s3::{
    S3Bucket, S3Entity, S3Event, S3EventRecord, S3Object, S3RequestParameters, S3UserIdentity,
};
use chrono::Utc;
use grapl_graph_descriptions::graph_description::*;
use grapl_observe::metric_reporter::MetricReporter;
use lambda_runtime::Context;
use log::*;
use rusoto_core::{HttpClient, Region};
use rusoto_s3::S3Client;
use rusoto_sqs::{SendMessageRequest, Sqs, SqsClient};
use sqs_lambda::cache::NopCache;
use sqs_lambda::event_decoder::PayloadDecoder;
use sqs_lambda::event_handler::EventHandler;
use sqs_lambda::local_sqs_service::local_sqs_service_with_options;
use sqs_lambda::local_sqs_service_options::LocalSqsServiceOptionsBuilder;
use sqs_lambda::sqs_completion_handler::CompletionPolicy;
use sqs_lambda::sqs_consumer::{ConsumePolicy, ConsumePolicyBuilder};
use std::io::Stdout;
use std::time::Duration;

const DEADLINE_LENGTH: i64 = 10_000; // 10,000 ms = 10 seconds

/// Runs the provided graph generator implementation locally.
///
/// Expects SOURCE_QUEUE_URL environment variable is set
pub(crate) async fn run_graph_generator_local<
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
    consume_policy: ConsumePolicyBuilder,
    completion_policy: CompletionPolicy,
    metric_reporter: MetricReporter<Stdout>,
) {
    let source_queue_url = std::env::var("SOURCE_QUEUE_URL").expect("SOURCE_QUEUE_URL");

    loop {
        let generator = generator.clone();
        let event_decoder = event_decoder.clone();

        if let Err(e) = initialize_local_service(
            &source_queue_url,
            generator,
            event_decoder,
            completion_policy.clone(),
            consume_policy.clone(),
            metric_reporter.clone(),
        )
        .await
        {
            error!("{}", e);
            std::thread::sleep(Duration::from_secs(2));
        }
    }
}

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
    queue_url: &str,
    generator: EH,
    event_decoder: ED,
    completion_policy: CompletionPolicy,
    consume_policy: ConsumePolicyBuilder,
    metric_reporter: MetricReporter<Stdout>,
) -> Result<(), Box<dyn std::error::Error>> {
    // ensure that local aws services (S3 and SQS) are ready and available
    let queue_name = queue_url.split("/").last().unwrap();
    grapl_config::wait_for_sqs(init_local_sqs_client(), queue_name).await?;
    grapl_config::wait_for_sqs(init_local_sqs_client(), "grapl-node-identifier-queue").await?;
    grapl_config::wait_for_s3(init_local_s3_client()).await?;

    // defines a deadline for the processing to complete by
    let service_execution_deadline = Context {
        deadline: Utc::now().timestamp_millis() + DEADLINE_LENGTH,
        ..Default::default()
    };

    // bucket for writing subgraphs containing local un-identified nodes
    // node-identifier will read from events emitted by this bucket to continue processing
    let destination_bucket = "local-grapl-unid-subgraphs-generated-bucket";

    let event_encoder = SubgraphSerializer::new(Vec::with_capacity(1024));

    let mut options_builder = LocalSqsServiceOptionsBuilder::default();
    options_builder.with_completion_policy(completion_policy);
    options_builder
        .with_consume_policy(consume_policy.build(Utc::now().timestamp_millis() + 10_000));

    /*
     * queue_url - The queue to be reading incoming log events from.
     * destination_bucket - The destination S3 bucket where completed, serialized subgraphs should be written to.
     * service_execution_deadline - defines the maximum length of time this service should spend trying to process the events
     * event_encoder - An event encoder (SubgraphSerializer) used to serialize the subgraphs created by the provided generator.
     * generator - An EventHandler that takes in log events, parses them, and generates subgraphs based on the logs provided.
     */
    local_sqs_service_with_options(
        queue_url,
        destination_bucket,
        service_execution_deadline,
        |_| init_local_s3_client(),
        init_local_s3_client(),
        init_local_sqs_client(),
        event_decoder,
        event_encoder,
        generator,
        NopCache {},
        metric_reporter,
        |_, event_result| debug!("{:?}", event_result),
        |bucket, key| local_emit_event(bucket, key),
        options_builder.build(),
    )
    .await?;
    Ok(())
}

/// Sends an SQS notification to the node-identifier sqs queue that a file has been
/// written to a particular bucket with key.
///
/// This is necessary for local testing as SNS cannot be used to connect
/// S3 and SQS together (as is the design of grapl in production)
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

    // publish to node-identifier SQS
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

/// Creates an SQS client for local testing.
///
/// Note: The custom region's endpoint address is set in docker-compose as the container to talk to
/// when interacting with the local sqs service. In other words, this endpoint does NOT
/// point to the actual SQS service.
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
