/*
THIS FILE WILL BE DELETED IN THE NEXT ~15 DAYS
Don't bother touching it!
~ wimax, July 11, 2022
*/
use clap::Parser;
use futures::StreamExt;
use grapl_tracing::setup_tracing;
use kafka::{
    config::{
        ConsumerConfig,
        ProducerConfig,
    },
    StreamProcessor,
    StreamProcessorError,
};
use rust_proto::graplinc::grapl::{
    api::graph::v1beta1::GraphDescription,
    pipeline::{
        v1beta1::{
            Metadata,
            RawLog,
        },
        v1beta2::Envelope,
    },
};
use sysmon_parser::SysmonEvent;
use tracing::instrument::WithSubscriber;

mod error;
mod models;

use crate::error::SysmonGeneratorError;

const SERVICE_NAME: &'static str = "sysmon-generator-legacy";

#[tokio::main]
async fn main() -> Result<(), SysmonGeneratorError> {
    let _guard = setup_tracing(SERVICE_NAME)?;

    handler().await
}

#[tracing::instrument]
async fn handler() -> Result<(), SysmonGeneratorError> {
    let consumer_config = ConsumerConfig::parse();
    let producer_config = ProducerConfig::parse();

    tracing::info!(
        message = "Configuring Kafka StreamProcessor",
        consumer_config = ?consumer_config,
        producer_config = ?producer_config,
    );

    // TODO: also construct a stream processor for retries

    let stream_processor = StreamProcessor::new(consumer_config, producer_config)?;

    tracing::info!(message = "Kafka StreamProcessor configured successfully");

    let stream = stream_processor.stream(event_handler);

    stream
        .for_each_concurrent(
            10, // TODO: make configurable?
            |res| async move {
                if let Err(e) = res {
                    // TODO: retry the message?
                    tracing::error!(
                        message = "Error processing Kafka message",
                        reason = %e,
                    );
                } else {
                    // TODO: collect some metrics
                    tracing::debug!(message = "Generated graph from sysmon event");
                }
            },
        )
        .with_current_subscriber()
        .await;

    Ok(())
}

async fn event_handler(
    event: Result<Envelope<RawLog>, StreamProcessorError>,
) -> Result<Option<Envelope<GraphDescription>>, SysmonGeneratorError> {
    let envelope = event?;
    let sysmon_event = SysmonEvent::from_str(std::str::from_utf8(
        envelope.inner_message.log_event.as_ref(),
    )?)?;

    match models::generate_graph_from_event(&sysmon_event)? {
        Some(graph_description) => Ok(Some(Envelope::new(
            Metadata::create_from(envelope.metadata),
            graph_description,
        ))),
        None => Ok(None),
    }
}
