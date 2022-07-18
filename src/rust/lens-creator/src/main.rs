#![allow(warnings)]

use clap::Parser;
use futures::{
    pin_mut,
    StreamExt,
};
use kafka::{
    config::{
        ConsumerConfig,
        ProducerConfig,
    },
    Consumer,
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
        lens_manager::v1beta1::client::LensManagerServiceClient,
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

use crate::{
    config::LensCreatorConfig,
    service::{
        LensCreator,
        LensCreatorError,
    },
};

mod config;
pub mod service;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (non_blocking, _guard) = tracing_appender::non_blocking(std::io::stdout());
    let service_config = LensCreatorConfig::parse();
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

    let lens_manager_client =
        LensManagerServiceClient::connect(service_config.lens_manager_client_url).await?;
    let lens_creator = LensCreator::new(lens_manager_client);

    let consumer_config = ConsumerConfig::parse();
    let producer_config = ProducerConfig::parse();

    handler(lens_creator, consumer_config, producer_config).await
}

#[tracing::instrument(skip(lens_creator))]
async fn handler(
    lens_creator: LensCreator,
    consumer_config: ConsumerConfig,
    producer_config: ProducerConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!(
        message = "configuring kafka stream processor",
        bootstrap_servers = %consumer_config.bootstrap_servers,
        consumer_group_name = %consumer_config.consumer_group_name,
        consumer_topic = %consumer_config.topic,
        producer_topic = %producer_config.topic,
    );

    // TODO: also construct a stream processor for retries

    let stream_processor: StreamProcessor<Envelope<ExecutionHit>, Envelope<LensUpdates>> =
        StreamProcessor::new(consumer_config, producer_config)?;

    tracing::info!(message = "kafka stream processor configured successfully",);

    let stream = stream_processor.stream::<_, _, StreamProcessorError>(
        move |event: Result<Envelope<ExecutionHit>, StreamProcessorError>| async move {
            let envelope = event.unwrap();
            let updates = lens_creator
                .handle_event(envelope.metadata.tenant_id, envelope.inner_message)
                .await;

            match updates {
                Ok(lens) => Ok(Some(Envelope::new(
                    Metadata::create_from(envelope.metadata),
                    lens,
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
        },
    );

    stream
        .for_each_concurrent(
            10, // TODO: make configurable?
            |event| async {
                let envelope = match event {
                    Ok(envelope) => envelope,
                    Err(e) => {
                        tracing::error!(message = "error while deserializing Envelope<ExecutionHit>", error = ? e);
                        return;
                    }
                };
                let res = lens_creator
                    .handle_event(envelope.metadata.tenant_id, envelope.inner_message)
                    .await;

                match res {
                    Ok(_) => tracing::info!(message = "event handled successfully", ),
                    Err(e) => tracing::error!(message = "error while handling event", error = ? e),
                }
            },
        )
        .with_current_subscriber()
        .await;

    Ok(())
}
