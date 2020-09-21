use crate::serialization::SubgraphSerializer;
use aws_lambda_events::event::sqs::SqsEvent;
use graph_descriptions::graph_description::*;
use grapl_config as config;
use lambda_runtime::error::HandlerError;
use lambda_runtime::Context;
use log::*;
use rusoto_core::Region;
use rusoto_s3::S3Client;
use rusoto_sqs::SqsClient;
use rusoto_sts::{StsAssumeRoleSessionCredentialsProvider, StsClient};
use sqs_lambda::event_decoder::PayloadDecoder;
use sqs_lambda::event_handler::EventHandler;
use std::collections::HashSet;
use std::str::FromStr;
use std::time::Duration;

/// Runs the graph generator on AWS
///
/// This function will assume the generator is to be run on an AWS lambda with services such as
/// S3 and SQS immediately available and accessible.
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

    let t = std::thread::spawn(move || {
        // lambda_runtime uses tokio::runtime::Runtime internally so we need to use tokio_compat::run_std if we want to invoke a new async task
        tokio_compat::run_std(async move {
            let source_queue_url = std::env::var("SOURCE_QUEUE_URL").expect("SOURCE_QUEUE_URL");
            info!("Queue Url: {}", source_queue_url);

            let bucket_prefix = std::env::var("BUCKET_PREFIX").expect("BUCKET_PREFIX");
            let destination_bucket = bucket_prefix + "-unid-subgraphs-generated-bucket";
            info!("Output events to: {}", destination_bucket);

            let region = config::region();
            let cache = config::event_cache().await;

            let initial_messages: Vec<_> = event.records.into_iter().map(map_sqs_message).collect();

            sqs_lambda::sqs_service::sqs_service(
                source_queue_url,
                initial_messages,
                destination_bucket,
                ctx,
                |region_str| init_production_s3_client(region_str),
                S3Client::new(region.clone()),
                SqsClient::new(region.clone()),
                event_decoder,
                SubgraphSerializer::new(Vec::with_capacity(1024)),
                generator,
                cache,
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
            completed_tx.send("Completed".to_owned()).unwrap();
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
