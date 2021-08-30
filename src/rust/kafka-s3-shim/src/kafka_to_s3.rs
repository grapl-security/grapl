use futures_util::{pin_mut, Stream, StreamExt};
use rdkafka::consumer::{CommitMode, Consumer, DefaultConsumerContext, StreamConsumer};
use rdkafka::error::KafkaError;
use rdkafka::message::BorrowedMessage;
use rdkafka::util::DefaultRuntime;
use rdkafka::Message;
use rusoto_s3::{S3Client, S3, PutObjectRequest};
use tokio::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};

fn generate_key() -> String {
    let cur_secs = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(n) => n.as_secs(),
        Err(_) => panic!("SystemTime before UNIX EPOCH!"),
    };

    let cur_day = cur_secs - (cur_secs % 86400);
    let capability = rand::random::<u128>();
    format!(
        "{day}/{seconds}/{capability}",
        day = cur_day,
        seconds = cur_secs,
        capability = capability,
    )
}

async fn upload_to_s3(
    payload: &[u8],
    destination_bucket: String,
    s3_client: &S3Client,
) -> Result<(), Box<dyn std::error::Error>> {
    s3_client.put_object(PutObjectRequest {
        acl: None,
        body: Some(payload.to_vec().into()),
        bucket: destination_bucket,
        key: "".to_string(),
        ..PutObjectRequest::default()
    }).await?;
    Ok(())
}

pub async fn kafka_to_s3_loop(
    destination_bucket: String,
    s3_client: S3Client,
    message_consumer: StreamConsumer<DefaultConsumerContext, DefaultRuntime>,
) -> Result<(), Box<dyn std::error::Error>> {
    let message_stream = message_consumer.stream();
    pin_mut!(message_stream);

    loop {
        while let Some(message) = message_stream.next().await {
            let message = match message {
                Ok(msg) => msg,
                Err(e) => {
                    tracing::error!(message="consumer error", error=%e);
                    tokio::time::sleep(Duration::from_millis(500)).await;
                    continue;
                }
            };
            let payload = match message.payload() {
                None => {
                    tracing::warn!(message = "Empty payload");
                    continue;
                }
                Some(payload) => payload,
            };

            if let Err(e) = upload_to_s3(payload, destination_bucket.clone(), &s3_client).await {
                tracing::error!(message="Failed to upload to S3", destination_bucket=%destination_bucket, error=%e);
                continue;
            };

            if let Err(e) = message_consumer.commit_message(&message, CommitMode::Sync) {
                tracing::error!(message="Failed to commit kafka message", destination_bucket=%destination_bucket, error=%e);
                continue;
            }
        }
        tokio::task::yield_now().await;
    }
}
