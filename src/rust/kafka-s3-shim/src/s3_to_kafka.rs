use async_trait::async_trait;
// use tokio_stream::StreamExt;
use futures_util::{pin_mut, Stream, StreamExt};
use grapl_config::env_helpers::FromEnv;
use rdkafka::config::FromClientConfig;
use rdkafka::consumer::{
    CommitMode, Consumer, ConsumerContext, DefaultConsumerContext, StreamConsumer,
};
use rdkafka::error::KafkaResult;
use rdkafka::message::{BorrowedMessage, OwnedMessage};
use rdkafka::producer::{FutureProducer, FutureRecord, Producer};
use rdkafka::util::DefaultRuntime;
use rdkafka::Message;
use rusoto_s3::{GetObjectRequest, S3Client, S3};
use rusoto_sqs::{
    DeleteMessageRequest, ReceiveMessageRequest, ReceiveMessageResult, Sqs, SqsClient,
};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tracing::{debug, error, info, warn, Instrument};

async fn receive_messages(
    queue_url: String,
    sqs_client: &SqsClient,
) -> Result<Vec<rusoto_sqs::Message>, Box<dyn std::error::Error>> {
    const WAIT_TIME_SECONDS: i64 = 20;
    let messages = sqs_client
        .receive_message(ReceiveMessageRequest {
            max_number_of_messages: Some(10),
            queue_url,
            visibility_timeout: Some(30),
            wait_time_seconds: Some(WAIT_TIME_SECONDS),
            ..Default::default()
        })
        .await?;

    let messages = messages.messages.unwrap_or_else(|| vec![]);

    Ok(messages)
}

async fn get_s3_object(
    s3: &S3Client,
    msg: rusoto_sqs::Message,
) -> Result<Option<Vec<u8>>, Box<dyn std::error::Error>> {
    let body = msg.body.as_ref().unwrap();
    debug!("Got body from message: {}", body);
    let event: serde_json::Value = serde_json::from_str(body)?;
    if let Some(Some(event_str)) = event.get("Event").map(serde_json::Value::as_str) {
        if event_str == "s3:TestEvent" {
            return Ok(None);
        }
    }
    let record = &event["Records"][0]["s3"];

    let bucket = record["bucket"]["name"].as_str().expect("bucket name");
    let key = record["object"]["key"].as_str().expect("object key");

    let inner_loop_span = tracing::trace_span!(
        "s3.retrieve_event",
        bucket=%bucket,
        key=%key,
    );
    let _enter = inner_loop_span.enter();

    debug!(message = "Retrieving S3 payload",);

    let s3_data = s3.get_object(GetObjectRequest {
        bucket: bucket.to_string(),
        key: key.to_string(),
        ..Default::default()
    });

    let s3_data = tokio::time::timeout(Duration::from_secs(3), s3_data);

    let s3_data = tokio::time::timeout(Duration::from_secs(3), s3_data).await???;

    let mut body = Vec::with_capacity(1024);
    s3_data
        .body
        .expect("Missing S3 body")
        .into_async_read()
        .read_to_end(&mut body)
        .await?;

    if body.is_empty() {
        warn!(message = "S3 object is empty",);
        return Ok(None);
    }
    Ok(Some(body))
}

async fn publish_to_kafka(
    topic: &str,
    producer: &FutureProducer,
    payload: &[u8],
) -> Result<(), Box<dyn std::error::Error>> {
    let record: FutureRecord<[u8], _> = FutureRecord {
        topic: &topic,
        partition: None,
        payload: Some(payload),
        key: None,
        timestamp: None,
        headers: None,
    };

    match producer.send(record, Duration::from_secs(3)).await {
        Ok((partition, offset)) => {
            tracing::debug!(
                message="Metric published",
                topic=%topic,
                partition=?partition,
                offset=?offset,
            );
        }
        Err((e, _)) => {
            tracing::error!(
                message="Failed to send message to kafka",
                topic=%topic,
                error=%e,
            );
            Err(e)?;
        }
    }
    producer.flush(Duration::from_secs(3));

    Ok(())
}

async fn s3_to_kafka(
    s3_client: &S3Client,
    sqs_client: &SqsClient,
    producer: &FutureProducer,
    destination_topic: &str,
    message: rusoto_sqs::Message,
    queue_url: String,
    receipt_handle: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let s3_bytes = match get_s3_object(&s3_client, message).await {
        Ok(s3_bytes) => s3_bytes,
        Err(_) => return Ok(()),
    };

    if let Some(s3_bytes) = s3_bytes {
        if publish_to_kafka(&destination_topic, &producer, &s3_bytes)
            .await
            .is_err()
        {
            return Ok(());
        };
    }

    sqs_client
        .delete_message(DeleteMessageRequest {
            queue_url,
            receipt_handle,
        })
        .await?;
    Ok(())
}

pub async fn s3_to_kafka_loop(
    destination_topic: String,
    queue_url: String,
    sqs_client: SqsClient,
    s3_client: S3Client,
    producer: FutureProducer,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut messages = receive_messages(queue_url.clone(), &sqs_client)
        .await?
        .into_iter();

    while let Some(message) = messages.next() {
        let receipt_handle = message
            .receipt_handle
            .clone()
            .expect("receipt_handle is guaranteed");

        let message_id = message
            .message_id
            .clone()
            .expect("message_id is guaranteed");
        let loop_span = tracing::trace_span!(
            "message-loop",
            queue_url=%queue_url,
            message_id=%message_id,
            receipt_handle=%receipt_handle,
        );

        if let Err(e) = s3_to_kafka(
            &s3_client,
            &sqs_client,
            &producer,
            &destination_topic,
            message,
            queue_url.clone(),
            receipt_handle.clone(),
        )
        .instrument(loop_span)
        .await
        {
            tracing::error!(
                message = "s3_to_kafka failed",
                error=%e,
            );
        };
    }

    Ok(())
}
