use clap::Parser;
use futures::StreamExt;
use kafka::{
    config::ConsumerConfig,
    Consumer,
};
use opentelemetry::{
    global,
    sdk::propagation::TraceContextPropagator,
};
use rust_proto::graplinc::grapl::{
    api::{
        lens_manager::v1beta1::client::LensManagerServiceClient,
        plugin_sdk::analyzers::v1beta1::messages::ExecutionHit,
    },
    pipeline::v1beta2::Envelope,
};
use tracing_subscriber::{
    prelude::*,
    EnvFilter,
};

use crate::{
    config::LensCreatorConfig,
    service::LensCreator,
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
        .with_service_name("lens-creator")
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

    handler(lens_creator, consumer_config).await
}

#[tracing::instrument(skip(lens_creator))]
async fn handler(
    lens_creator: LensCreator,
    consumer_config: ConsumerConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!(
        message = "configuring kafka stream processor",
        bootstrap_servers = %consumer_config.bootstrap_servers,
        consumer_group_name = %consumer_config.consumer_group_name,
        consumer_topic = %consumer_config.topic,
    );

    // TODO: also construct a stream processor for retries

    let consumer: Consumer<Envelope<ExecutionHit>> = Consumer::new(consumer_config)?;

    tracing::info!(message = "kafka consumer configured successfully",);

    let stream = consumer.stream();

    stream
        .for_each_concurrent(10, |event| async {
            let envelope = match event {
                Ok(event) => event,
                Err(e) => {
                    tracing::error!(
                        message="error while consuming event: {}",
                        error=?e,
                    );
                    return;
                }
            };
            if let Err(e) = lens_creator
                .handle_event(envelope.metadata.tenant_id, envelope.inner_message)
                .await
            {
                tracing::error!(
                    message="error while processing event",
                    error=?e,
                );
            }
        })
        .await;

    Ok(())
}
