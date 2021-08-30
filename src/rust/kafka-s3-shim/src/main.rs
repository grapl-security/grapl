#![allow(warnings)]

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

use async_trait::async_trait;
use futures_util::{pin_mut, Stream, StreamExt};
use rdkafka::config::{FromClientConfig, RDKafkaLogLevel};
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
use tokio::io::AsyncReadExt;
use tracing::{debug, error, info, warn, Instrument};

use grapl_config::env_helpers::FromEnv;

use crate::kafka_to_s3::kafka_to_s3_loop;
use crate::s3_to_kafka::s3_to_kafka_loop;

mod kafka_to_s3;
mod s3_to_kafka;


fn producer_init() -> Result<FutureProducer, Box<dyn std::error::Error>> {
    let client_id = std::env::var("KAFKA_CLIENT_ID")?;
    let brokers = std::env::var("KAFKA_BROKERS")?;
    let mut client_config = rdkafka::ClientConfig::new();
    client_config
        .set("client.id", &client_id)
        .set("queue.buffering.max.ms", "0")
        .set("bootstrap.servers", &brokers);
    tracing::info!(config=?client_config, message="Created Producer ClientConfig");

    let producer: FutureProducer = FutureProducer::from_config(&client_config)?;
    Ok(producer)
}

fn consumer_init(
) -> Result<StreamConsumer<DefaultConsumerContext, DefaultRuntime>, Box<dyn std::error::Error>>
{
    let group_id = std::env::var("KAFKA_GROUP_ID")?;
    let client_id = std::env::var("KAFKA_CLIENT_ID")?;
    let source_topic = std::env::var("SOURCE_TOPIC")?;
    let brokers = std::env::var("KAFKA_BROKERS")?;

    let mut client_config = rdkafka::ClientConfig::new();
    client_config
        .set("group.id", group_id)
        .set("client.id", client_id)
        .set("bootstrap.servers", &brokers)
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "true")
        .set("max.poll.interval.ms", "10000")
        .set("request.timeout.ms", "1000")
        .set("auto.offset.reset", "latest")
        .set_log_level(RDKafkaLogLevel::Debug);
    tracing::info!(config=?client_config, message="Created Consumer ClientConfig");

    let consumer = StreamConsumer::from_config(&client_config)?;
    consumer
        .subscribe(&[&source_topic])
        .expect("Can't subscribe to specified topic");
    Ok(consumer)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // based on environment, either do s3 to kafka or kafka to s3
    // if you need s3 to s3, just use sqs-executor
    // if you need kafka to kafka, use the kafka-executor

    let sqs_client = SqsClient::from_env();
    let s3_client = S3Client::from_env();

    if let Ok(destination_topic) = std::env::var("DEST_KAFKA_TOPIC") {
        let source_queue_url = std::env::var("SOURCE_QUEUE_URL")?;

        let producer = producer_init()?;

        s3_to_kafka_loop(
            destination_topic,
            source_queue_url,
            sqs_client,
            s3_client,
            producer,
        )
        .await?;
    } else {
        let destination_bucket = std::env::var("DESTINATION_BUCKET")?;

        let message_consumer = consumer_init()?;

        kafka_to_s3_loop(destination_bucket, s3_client, message_consumer).await?;
    }

    Ok(())
}
