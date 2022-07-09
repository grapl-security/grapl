use clap::Parser;
use futures::StreamExt;
use grapl_config::env_helpers::FromEnv;
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
use rusoto_dynamodb::DynamoDbClient;
use rust_proto::graplinc::grapl::{
    api::graph::v1beta1::{
        GraphDescription,
        IdentifiedGraph,
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

mod dynamic_sessiondb;
mod error;
mod node_identifier;
mod sessiondb;
mod sessions;

use crate::{
    dynamic_sessiondb::NodeDescriptionIdentifier,
    error::NodeIdentifierError,
    node_identifier::NodeIdentifier,
    sessiondb::SessionDb,
};

#[tokio::main]
async fn main() -> Result<(), NodeIdentifierError> {
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
async fn handler() -> Result<(), NodeIdentifierError> {
    let dynamo = DynamoDbClient::from_env();
    let dyn_session_db = SessionDb::new(
        dynamo.clone(),
        std::env::var("GRAPL_DYNAMIC_SESSION_TABLE")?,
    );
    let node_identifier = NodeIdentifier::new(NodeDescriptionIdentifier::new(dyn_session_db, true));

    let consumer_config = ConsumerConfig::parse();
    let producer_config = ProducerConfig::parse();

    tracing::info!(
        message = "Configuring Kafka StreamProcessor",
        consumer_config = ?consumer_config,
        producer_config = ?producer_config,
    );

    // TODO: also construct a stream processor for retries

    let stream_processor: StreamProcessor<Envelope<GraphDescription>, Envelope<IdentifiedGraph>> =
        StreamProcessor::new(consumer_config, producer_config)?;

    tracing::info!(message = "Kafka StreamProcessor configured successfully");

    let stream = stream_processor.stream::<_, _, StreamProcessorError>(
        move |event: Result<Envelope<GraphDescription>, StreamProcessorError>| {
            let identifier = node_identifier.clone();
            async move {
                let envelope = event?;

                match identifier.handle_event(envelope.inner_message).await {
                    Ok(identified_graph) => Ok(Some(Envelope::new(
                        Metadata::create_from(envelope.metadata),
                        identified_graph,
                    ))),
                    Err(e) => match e {
                        Ok((_, e)) => {
                            match e {
                                NodeIdentifierError::AttributionFailure => {
                                    tracing::warn!(
                                        message = "failed to attribute",
                                        error = %e,
                                    );
                                    // TODO: write message to retry topic here
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
                            NodeIdentifierError::EmptyGraph => {
                                tracing::warn!(message = "identified subgraph is empty",);
                                Ok(None)
                            }
                            _ => {
                                tracing::error!(
                                    message = "unexpected error",
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
