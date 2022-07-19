use std::sync::Arc;

use clap::Parser;
use dgraph_tonic::Client as DgraphClient;
use futures::StreamExt;
use grapl_config::env_helpers::FromEnv;
use grapl_tracing::setup_tracing;
use kafka::{
    config::{
        ConsumerConfig,
        ProducerConfig,
    },
    StreamProcessor,
    StreamProcessorError,
};
use rusoto_dynamodb::DynamoDbClient;
use rust_proto::graplinc::grapl::{
    api::graph::v1beta1::{
        IdentifiedGraph,
        MergedGraph,
    },
    pipeline::{
        v1beta1::Metadata,
        v1beta2::Envelope,
    },
};
use tokio::sync::Mutex;
use tracing::instrument::WithSubscriber;

use crate::{
    reverse_resolver::ReverseEdgeResolver,
    service::{
        GraphMerger,
        GraphMergerError,
    },
};

pub mod reverse_resolver;
pub mod service;
pub mod upsert_util;
pub mod upserter;

const SERVICE_NAME: &'static str = "graph-merger";

#[tokio::main]
async fn main() -> Result<(), GraphMergerError> {
    let _guard = setup_tracing(SERVICE_NAME)?;

    let mg_alphas = grapl_config::mg_alphas();
    let dynamo = DynamoDbClient::from_env();
    let reverse_edge_resolver = ReverseEdgeResolver::new(dynamo, 1000);

    tracing::debug!(
        mg_alphas =? &mg_alphas,
        message = "Connecting to mg_alphas"
    );

    let graph_merger = GraphMerger::new(DgraphClient::new(mg_alphas)?, reverse_edge_resolver);

    let consumer_config = ConsumerConfig::parse();
    let producer_config = ProducerConfig::parse();

    handler(
        Arc::new(Mutex::new(graph_merger)),
        consumer_config,
        producer_config,
    )
    .await
}

#[tracing::instrument(skip(graph_merger))]
async fn handler(
    graph_merger: Arc<Mutex<GraphMerger>>,
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

    let stream_processor: StreamProcessor<Envelope<IdentifiedGraph>, Envelope<MergedGraph>> =
        StreamProcessor::new(consumer_config, producer_config)?;

    tracing::info!(message = "kafka stream processor configured successfully",);

    let stream = stream_processor.stream::<_, _, StreamProcessorError>(
        move |event: Result<Envelope<IdentifiedGraph>, StreamProcessorError>| {
            let graph_merger = graph_merger.clone();
            async move {
                let envelope = event?;
                match graph_merger
                    .lock()
                    .await
                    .handle_event(envelope.inner_message)
                    .await
                {
                    Ok(merged_graph) => Ok(Some(Envelope::new(
                        Metadata::create_from(envelope.metadata),
                        merged_graph,
                    ))),
                    Err(e) => match e {
                        Ok((_, e)) => {
                            match e {
                                GraphMergerError::Unexpected(ref reason) => {
                                    tracing::error!(
                                        message = "unexpected error",
                                        reason = %reason,
                                        error = %e,
                                    );
                                    // TODO: write message to failed topic here
                                    Err(StreamProcessorError::from(e))
                                }
                                _ => {
                                    tracing::error!(
                                        mesage = "unexpected error",
                                        error = %e,
                                    );
                                    // TODO: write message to failed topic here
                                    Err(StreamProcessorError::from(e))
                                }
                            }
                        }
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
