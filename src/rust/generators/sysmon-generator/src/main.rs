use clap::Parser;
use futures::StreamExt;
use kafka::{
    config::{
        ConsumerConfig,
        ProducerConfig,
    },
    StreamProcessor,
    StreamProcessorError,
};
use opentelemetry::{
    global,
    sdk::propagation::TraceContextPropagator,
};
use rust_proto_new::graplinc::grapl::{
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
use tracing_subscriber::{
    prelude::*,
    EnvFilter,
};

mod error;
mod models;

use crate::error::SysmonGeneratorError;

#[tokio::main]
async fn main() -> Result<(), SysmonGeneratorError> {
    let (non_blocking, _guard) = tracing_appender::non_blocking(std::io::stdout());

    // initialize json logging layer
    let log_layer = tracing_subscriber::fmt::layer()
        .json()
        .with_writer(non_blocking);

    // initialize tracing layer
    global::set_text_map_propagator(TraceContextPropagator::new());
    let tracer = opentelemetry_jaeger::new_pipeline()
        .with_service_name("pipeline-ingress")
        .install_batch(opentelemetry::runtime::Tokio)?;

    // register a subscriber
    let filter = EnvFilter::from_default_env();
    tracing_subscriber::registry()
        .with(filter)
        .with(log_layer)
        .with(tracing_opentelemetry::layer().with_tracer(tracer))
        .init();

    tracing::info!("logger configured successfully");

    handler().await
}

#[tracing::instrument]
async fn handler() -> Result<(), SysmonGeneratorError> {
    let consumer_config = ConsumerConfig::parse();
    let consumer_topic = "raw-logs".to_string();
    let producer_config = ProducerConfig::parse();
    let producer_topic = "generated-graphs".to_string();

    tracing::info!(
        message = "configuring kafka stream processor",
        consumer_config = ?consumer_config,
        consumer_topic = %consumer_topic,
        producer_config = ?producer_config,
        producer_topic = %producer_topic,
    );

    // TODO: also construct a stream processor for retries

    let stream_processor = StreamProcessor::new(
        consumer_config,
        consumer_topic,
        producer_config,
        producer_topic,
    )?;

    tracing::info!(message = "kafka stream processor configured successfully",);
    tracing::info!("starting up!");

    let stream = stream_processor.stream(event_handler)?;

    stream
        .for_each_concurrent(
            10, // TODO: make configurable?
            |res| async move {
                if let Err(e) = res {
                    // TODO: retry the message?
                    tracing::error!(
                        message = "error processing kafka message",
                        reason = %e,
                    );
                } else {
                    // TODO: collect some metrics
                    tracing::debug!(message = "generated graph from sysmon event");
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
