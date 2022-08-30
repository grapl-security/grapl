#![allow(warnings)]
use std::sync::Arc;

use clap::Parser;
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
use rust_proto::{
    graplinc::grapl::{
        api::{
            graph::v1beta1::IdentifiedGraph,
            graph_mutation::v1beta1::client::GraphMutationClient,
            plugin_sdk::analyzers::v1beta1::messages::Updates,
        },
        pipeline::v1beta1::Envelope,
    },
    protocol::{
        endpoint::Endpoint,
        service_client::Connectable,
    },
};
use tokio::sync::Mutex;
use tracing::instrument::WithSubscriber;

use crate::{
    config::GraphMergerConfig,
    service::{
        GraphMerger,
        GraphMergerError,
    },
};

pub mod config;
pub mod service;

const SERVICE_NAME: &'static str = "graph-merger";

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let _guard = setup_tracing(SERVICE_NAME)?;

    let service_config = GraphMergerConfig::parse();
    let graph_mutation_client = GraphMutationClient::connect(Endpoint::from_shared(
        service_config.graph_mutation_client_url,
    )?)
    .await?;
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
) -> eyre::Result<()> {
    tracing::info!(
        message = "configuring kafka stream processor",
        bootstrap_servers = %consumer_config.bootstrap_servers,
        consumer_group_name = %consumer_config.consumer_group_name,
        consumer_topic = %consumer_config.topic,
        producer_topic = %producer_config.topic,
    );

    // TODO: also construct a stream processor for retries

    let stream_processor: StreamProcessor<IdentifiedGraph, Updates> =
        StreamProcessor::new(consumer_config, producer_config)?;

    tracing::info!(message = "kafka stream processor configured successfully",);

    let stream = stream_processor.stream::<_, _, StreamProcessorError>(move |event| {
        let mut graph_merger = graph_merger.clone();
        async move {
            let envelope = event?;
            let tenant_id = envelope.tenant_id();
            let trace_id = envelope.trace_id();
            let event_source_id = envelope.event_source_id();

            tracing::debug!("received kafka message");

            match graph_merger
                .handle_event(tenant_id, envelope.inner_message())
                .await
            {
                Ok(updates) => Ok(Some(Envelope::new(
                    tenant_id,
                    trace_id,
                    event_source_id,
                    updates,
                ))),
                Err(e) => match e {
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
                },
            }
        }
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
