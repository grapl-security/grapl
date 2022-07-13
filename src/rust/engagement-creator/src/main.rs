#![allow(warnings)]

use clap::Parser;
use futures::{pin_mut, StreamExt};
use kafka::{config::{
    ConsumerConfig,
    ProducerConfig,
}, Consumer, StreamProcessor, StreamProcessorError};
use opentelemetry::{
    global,
    sdk::propagation::TraceContextPropagator,
};
use rust_proto::graplinc::grapl::{
    api::{
        graph::v1beta1::IdentifiedGraph,
        graph_mutation::v1beta1::client::GraphMutationClient,
        plugin_sdk::analyzers::v1beta1::messages::ExecutionHit,
    },
    pipeline::{
        v1beta1::Metadata,
        v1beta2::Envelope,
    },
};
use tracing::instrument::WithSubscriber;
use tracing_subscriber::{
    prelude::*,
    EnvFilter,
};
use crate::config::EngagementCreatorConfig;

use crate::service::{
    EngagementCreator,
    EngagementCreatorError,
};

pub mod service;
mod config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (non_blocking, _guard) = tracing_appender::non_blocking(std::io::stdout());
    let service_config = EngagementCreatorConfig::parse();
    // initialize json logging layer
    let log_layer = tracing_subscriber::fmt::layer()
        .json()
        .with_writer(non_blocking);

    // initialize tracing layer
    global::set_text_map_propagator(TraceContextPropagator::new());
    let tracer = opentelemetry_jaeger::new_pipeline()
        .with_service_name("graph-merger")
        .install_batch(opentelemetry::runtime::Tokio)?;

    // register a subscriber
    let filter = EnvFilter::from_default_env();
    tracing_subscriber::registry()
        .with(filter)
        .with(log_layer)
        .with(tracing_opentelemetry::layer().with_tracer(tracer))
        .init();

    tracing::info!("logger configured successfully");

    let graph_mutation_client =
        GraphMutationClient::connect(service_config.graph_mutation_client_url).await?;
    let engagement_creator = EngagementCreator::new(graph_mutation_client);

    let consumer_config = ConsumerConfig::parse();

    handler(engagement_creator, consumer_config).await
}

#[tracing::instrument(skip(engagement_creator))]
async fn handler(
    engagement_creator: EngagementCreator,
    consumer_config: ConsumerConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!(
        message = "configuring kafka stream processor",
        bootstrap_servers = %consumer_config.bootstrap_servers,
        consumer_group_name = %consumer_config.consumer_group_name,
        consumer_topic = %consumer_config.topic,
    );

    // TODO: also construct a stream processor for retries

    let stream_consumer: Consumer<Envelope<ExecutionHit>> = Consumer::new(consumer_config)?;

    tracing::info!(message = "kafka stream processor configured successfully",);

    let stream = stream_consumer.stream()?;


    stream
        .for_each_concurrent(
            10, // TODO: make configurable?
            |event| async {
                let envelope = match event {
                    Ok(envelope) => envelope,
                    Err(e) => {
                        tracing::error!(message = "error while deserializing Envelope<ExecutionHit>", error = ?e);
                        return
                    }
                };
                let res = engagement_creator
                    .handle_event(envelope.metadata.tenant_id, envelope.inner_message)
                    .await;

                match res {
                    Ok(_) => tracing::info!(message = "event handled successfully",),
                    Err(e) => tracing::error!(message = "error while handling event", error = ?e),
                }
            },
        )
        .with_current_subscriber()
        .await;

    Ok(())
}
