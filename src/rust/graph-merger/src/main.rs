use clap::Parser;
use figment::{
    providers::Env,
    Figment,
};
use futures::{
    FutureExt,
    StreamExt,
};
use grapl_tracing::setup_tracing;
use kafka::{
    config::{
        ConsumerConfig,
        ProducerConfig,
    },
    StreamProcessor,
};
use rust_proto::graplinc::grapl::api::{
    client::Connect,
    graph::v1beta1::IdentifiedGraph,
    graph_mutation::v1beta1::client::GraphMutationClient,
    plugin_sdk::analyzers::v1beta1::messages::Update,
};
use tracing::instrument::WithSubscriber;

use crate::service::{
    GraphMerger,
    GraphMergerError,
};

pub mod service;

const SERVICE_NAME: &'static str = "graph-merger";

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let _guard = setup_tracing(SERVICE_NAME)?;

    let graph_mutation_client_config = Figment::new()
        .merge(Env::prefixed("GRAPH_MUTATION_CLIENT_"))
        .extract()?;
    let graph_mutation_client = GraphMutationClient::connect(graph_mutation_client_config).await?;
    let graph_merger = GraphMerger::new(graph_mutation_client);

    let consumer_config = ConsumerConfig::parse();
    let producer_config = ProducerConfig::parse();

    handler(graph_merger, consumer_config, producer_config).await
}

#[tracing::instrument(skip(graph_merger, consumer_config, producer_config))]
async fn handler(
    graph_merger: GraphMerger,
    consumer_config: ConsumerConfig,
    producer_config: ProducerConfig,
) -> eyre::Result<()> {
    tracing::info!(
        message = "configuring kafka stream processor",
        bootstrap_servers = %consumer_config.bootstrap_servers,
        consumer_group_name = %consumer_config.consumer_group_name,
        consumer_topic = %consumer_config.topic,
        producer_topic = %producer_config.topic,
    );

    // TODO: also construct a stream processor for retries

    let stream_processor: StreamProcessor<IdentifiedGraph, Update> =
        StreamProcessor::new(consumer_config, producer_config)?;

    tracing::info!(message = "kafka stream processor configured successfully",);

    let stream = stream_processor.stream::<_, _, GraphMergerError>(move |event| {
        graph_merger
            .clone()
            .handle_event(event)
            .into_stream()
            .flat_map(|results| futures::stream::iter(results))
    });

    stream
        .for_each(|res| async move {
            if let Err(e) = res {
                // TODO: retry the message?
                tracing::error!(
                    message = "error processing kafka message",
                    reason = %e,
                );
            } else {
                // TODO: collect some metrics
                tracing::debug!(message = "merged identified graph successfully");
            }
        })
        .with_current_subscriber()
        .await;

    Ok(())
}
