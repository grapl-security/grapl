use std::sync::Arc;

use dgraph_tonic::Client as DgraphClient;
use futures::StreamExt;
use grapl_config::env_helpers::FromEnv;
use kafka::{
    StreamProcessor,
    StreamProcessorError,
};
use opentelemetry::{
    global,
    sdk::propagation::TraceContextPropagator,
};
use rusoto_dynamodb::DynamoDbClient;
use rust_proto_new::graplinc::grapl::{
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
use tracing_subscriber::{
    prelude::*,
    EnvFilter,
};

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

#[tokio::main]
async fn main() -> Result<(), GraphMergerError> {
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

    let mg_alphas = grapl_config::mg_alphas();
    let dynamo = DynamoDbClient::from_env();
    let reverse_edge_resolver = ReverseEdgeResolver::new(dynamo, 1000);

    tracing::debug!(
        mg_alphas =? &mg_alphas,
        message = "Connecting to mg_alphas"
    );

    let graph_merger = GraphMerger::new(DgraphClient::new(mg_alphas)?, reverse_edge_resolver);

    handler(Arc::new(Mutex::new(graph_merger))).await
}

#[tracing::instrument(skip(graph_merger))]
async fn handler(graph_merger: Arc<Mutex<GraphMerger>>) -> Result<(), GraphMergerError> {
    let bootstrap_servers = std::env::var("KAFKA_BOOTSTRAP_SERVERS")?;
    let sasl_username = std::env::var("KAFKA_SASL_USERNAME")?;
    let sasl_password = std::env::var("KAFKA_SASL_PASSWORD")?;
    let consumer_group_name = std::env::var("GRAPH_MERGER_CONSUMER_GROUP")?;
    let consumer_topic = "identified-graphs".to_string();
    let producer_topic = "merged-graphs".to_string();

    tracing::info!(
        message = "configuring kafka stream processor",
        bootstrap_servers = %bootstrap_servers,
        consumer_group_name = %consumer_group_name,
        consumer_topic = %consumer_topic,
        producer_topic = %producer_topic,
    );

    // TODO: also construct a stream processor for retries

    let stream_processor: StreamProcessor<Envelope<IdentifiedGraph>, Envelope<MergedGraph>> =
        StreamProcessor::new(
            bootstrap_servers.clone(),
            sasl_username,
            sasl_password,
            consumer_group_name.clone(),
            consumer_topic.clone(),
            producer_topic.clone(),
        )?;

    tracing::info!(
        message = "kafka stream processor configured successfully",
        bootstrap_servers = %bootstrap_servers,
        consumer_group_name = %consumer_group_name,
        consumer_topic = %consumer_topic,
        producer_topic = %producer_topic,
    );
    tracing::info!("starting up!");

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
