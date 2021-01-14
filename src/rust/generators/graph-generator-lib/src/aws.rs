use crate::serialization::SubgraphSerializer;
use aws_lambda_events::event::sqs::SqsEvent;
use grapl_config as config;
use grapl_graph_descriptions::graph_description::*;
use grapl_observe::metric_reporter::MetricReporter;
use lambda_runtime::error::HandlerError;
use log::*;
use rusoto_core::Region;
use rusoto_s3::S3Client;
use rusoto_sqs::SqsClient;
use rusoto_sts::{StsAssumeRoleSessionCredentialsProvider, StsClient};
use sqs_lambda::event_decoder::PayloadDecoder;
use sqs_lambda::event_handler::EventHandler;
use sqs_lambda::sqs_completion_handler::CompletionPolicy;
use sqs_lambda::sqs_consumer::{ConsumePolicy, ConsumePolicyBuilder};
use std::collections::HashSet;
use std::io::Stdout;
use std::str::FromStr;
use std::sync::mpsc::SyncSender;
use std::time::Duration;

/// Runs the graph generator on AWS
///
/// This function will assume the generator is running on an AWS lambda with services such as
/// S3 and SQS immediately available and accessible.
pub(crate) fn run_graph_generator_aws<
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
    lambda_runtime::lambda!(|event, context| {
        let consume_policy = consume_policy.clone();
        lambda_handler(
            event,
            consume_policy.build(context),
            completion_policy.clone(),
            generator.clone(),
            event_decoder.clone(),
            metric_reporter.clone(),
        )
    })
}

fn lambda_handler<
    IE: Send + Sync + Clone + 'static,
    EH: EventHandler<InputEvent = IE, OutputEvent = Graph, Error = sqs_lambda::error::Error>
        + Send
        + Sync
        + Clone
        + 'static,
    ED: PayloadDecoder<IE> + Send + Sync + Clone + 'static,
>(
    event: SqsEvent,
    consume_policy: ConsumePolicy,
    completion_policy: CompletionPolicy,
    generator: EH,
    event_decoder: ED,
    metric_reporter: MetricReporter<Stdout>,
) -> Result<(), HandlerError> {
    info!("Handling event");

    // tracks the SQS message ids in this batch of events
    let mut initial_events: HashSet<String> = event
        .records
        .iter()
        .map(|event| event.message_id.clone().unwrap())
        .collect();

    info!("Initial Events {:?}", initial_events);

    let (tx, rx) = std::sync::mpsc::sync_channel(10);

    let t = std::thread::spawn(move || {
        // lambda_runtime uses tokio::runtime::Runtime internally so we need to use
        // tokio_compat::run_std if we want to invoke a new async task
        tokio_compat::run_std(run_async_generator_handler(
            event,
            consume_policy,
            completion_policy,
            generator,
            event_decoder,
            tx,
            metric_reporter,
        ));
    });

    info!("Checking acks");

    // TODO: Change this from using String to using proper Enums to more semantically convey meaning
    for received_message in &rx {
        info!("Acking event: {}", &received_message);

        initial_events.remove(&received_message);

        if received_message == "Completed" {
            // If we're done go ahead and try to clear out any remaining
            while let Ok(r) = rx.recv_timeout(Duration::from_millis(100)) {
                initial_events.remove(&r);
            }

            break;
        }
    }

    t.join();

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

/// Grabs initial events and starts a short-lived service that listens to SQS messages and processes them.
async fn run_async_generator_handler<
    IE: Send + Sync + Clone + 'static,
    EH: EventHandler<InputEvent = IE, OutputEvent = Graph, Error = sqs_lambda::error::Error>
        + Send
        + Sync
        + Clone
        + 'static,
    ED: PayloadDecoder<IE> + Send + Sync + Clone + 'static,
>(
    event: SqsEvent,
    consume_policy: ConsumePolicy,
    completion_policy: CompletionPolicy,
    generator: EH,
    event_decoder: ED,
    tx: SyncSender<String>,
    metric_reporter: MetricReporter<Stdout>,
) {
    let source_queue_url = std::env::var("SOURCE_QUEUE_URL").expect("SOURCE_QUEUE_URL");

    info!("Queue Url: {}", source_queue_url);

    let bucket_prefix = std::env::var("BUCKET_PREFIX").expect("BUCKET_PREFIX");
    let destination_bucket = format!("{}-unid-subgraphs-generated-bucket", bucket_prefix);

    info!("Output events to: {}", destination_bucket);

    let region = config::region();
    let cache = config::event_cache().await;

    let initial_messages: Vec<_> = event.records.into_iter().map(map_sqs_message).collect();

    let sqs_tx = tx.clone();

    sqs_lambda::sqs_service::sqs_service(
        source_queue_url,
        initial_messages,
        destination_bucket,
        consume_policy,
        completion_policy,
        |region_str| init_production_s3_client(region_str),
        S3Client::new(region.clone()),
        SqsClient::new(region.clone()),
        event_decoder,
        SubgraphSerializer::new(Vec::with_capacity(1024)),
        generator,
        cache,
        metric_reporter,
        move |_, result: Result<String, String>| report_sqs_service_result(result, &sqs_tx),
        move |bucket, key| async move {
            info!("Emitted event to {} {}", bucket, key);
            Ok(())
        },
    )
    .await
    .expect("service failed");

    tx.send("Completed".to_owned()).unwrap();
}

fn report_sqs_service_result(result: Result<String, String>, tx: &SyncSender<String>) {
    match result {
        Ok(success_message) => {
            info!(
                "Handled an event, which was successfully deleted: {}",
                &success_message
            );
            tx.send(success_message).unwrap();
        }
        Err(error_message) => {
            info!(
                "Handled an event, though we failed to delete it: {}",
                &error_message
            );
            tx.send(error_message).unwrap();
        }
    }
}

/// Maps an [aws_lambda_events::event::sqs::SqsMessage] into a [rusoto_sqs::Message] for easier processing
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

/// Creates an S3 client used when running Grapl on AWS
fn init_production_s3_client(region_str: String) -> S3Client {
    let region = Region::from_str(&region_str).expect("region_str");

    match std::env::var("ASSUME_ROLE") {
        Ok(role) => {
            let provider = StsAssumeRoleSessionCredentialsProvider::new(
                StsClient::new(region.clone()),
                role,
                "default".to_owned(),
                None,
                None,
                None,
                None,
            );
            S3Client::new_with(
                rusoto_core::request::HttpClient::new().expect("Failed to create HTTP client"),
                provider,
                region,
            )
        }
        _ => S3Client::new(region),
    }
}
