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
    pipeline::v1beta1::{
        Envelope,
        RawLog,
    },
};
use tracing::instrument::WithSubscriber;

mod error;
mod models;
use sysmon_generator::api::expect_one_event;

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
    let tenant_id = envelope.tenant_id();
    let trace_id = envelope.trace_id();
    let event_source_id = envelope.event_source_id();
    let raw_log = envelope.inner_message().log_event();

    let input_utf8 = std::str::from_utf8(raw_log.as_ref())?;
    let events: Vec<_> = sysmon_parser::parse_events(input_utf8).collect();
    let sysmon_event = expect_one_event(events)?;

    match models::generate_graph_from_event(&sysmon_event)? {
        Some(graph_description) => Ok(Some(Envelope::new(
            tenant_id,
            trace_id,
            event_source_id,
            graph_description,
        ))),
        None => Ok(None),
    }
}
