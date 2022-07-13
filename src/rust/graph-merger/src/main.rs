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
use rust_proto::graplinc::grapl::{
    api::{
        graph::v1beta1::IdentifiedGraph,
        graph_mutation::v1beta1::client::GraphMutationClient,
        plugin_sdk::analyzers::v1beta1::messages::Updates,
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
use crate::config::GraphMergerConfig;

use crate::service::{
    GraphMerger,
    GraphMergerError,
};

pub mod service;
mod config;

#[tokio::main]
async fn main() -> Result<(), GraphMergerError> {
    let (non_blocking, _guard) = tracing_appender::non_blocking(std::io::stdout());
    let service_config = GraphMergerConfig::parse();
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
    let graph_merger = GraphMerger::new(graph_mutation_client);

    let consumer_config = ConsumerConfig::parse();
    let producer_config = ProducerConfig::parse();

    handler(graph_merger, consumer_config, producer_config).await
}

#[tracing::instrument(skip(graph_merger))]
async fn handler(
    graph_merger: GraphMerger,
    consumer_config: ConsumerConfig,
    producer_config: ProducerConfig,
) -> Result<(), GraphMergerError> {
    tracing::info!(
        message = "configuring kafka stream processor",
        bootstrap_servers = %consumer_config.bootstrap_servers,
        consumer_group_name = %consumer_config.consumer_group_name,
        consumer_topic = %consumer_config.topic,
        producer_topic = %producer_config.topic,
    );

    // TODO: also construct a stream processor for retries

    let stream_processor: StreamProcessor<Envelope<IdentifiedGraph>, Envelope<Updates>> =
        StreamProcessor::new(consumer_config, producer_config)?;

    tracing::info!(message = "kafka stream processor configured successfully",);

    let stream = stream_processor.stream::<_, _, StreamProcessorError>(
        move |event: Result<Envelope<IdentifiedGraph>, StreamProcessorError>| {
            let mut graph_merger = graph_merger.clone();
            async move {
                let envelope = event?;
                match graph_merger
                    .handle_event(envelope.metadata.tenant_id, envelope.inner_message)
                    .await
                {
                    Ok(merged_graph) => Ok(Some(Envelope::new(
                        Metadata::create_from(envelope.metadata),
                        merged_graph,
                    ))),

                    Err(e) => match e {
                        GraphMergerError::Unexpected(reason) => {
                            tracing::warn!(
                                message = "unexpected error",
                                reason = %reason,
                            );
                            Ok(None)
                        }
                        _ => {
                            tracing::error!(
                                message = "unknown error",
                                error = %e,
                            );
                            Err(StreamProcessorError::from(e))
                        }
                    },
                }
            }
        },
    )?;

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
                    tracing::debug!(message = "identified graph from graph description");
                }
            },
        )
        .with_current_subscriber()
        .await;

    Ok(())
}
