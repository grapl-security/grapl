use opentelemetry::{
    global,
    sdk::propagation::TraceContextPropagator,
};
use rust_proto::graplinc::grapl::{
    api::graph::v1beta1::GraphDescription,
    pipeline::{
        v1beta1::RawLog,
        v1beta2::Envelope,
    },
};
use thiserror::Error;
use tracing_subscriber::{
    prelude::*,
    EnvFilter,
};

mod parsers;

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum OsqueryGeneratorError {
    #[error("error configuring tracing {0}")]
    TraceError(#[from] opentelemetry::trace::TraceError),
}

#[tokio::main]
#[tracing::instrument]
async fn main() -> Result<(), OsqueryGeneratorError> {
    let (non_blocking, _guard) = tracing_appender::non_blocking(std::io::stdout());

    // initialize json logging layer
    let log_layer = tracing_subscriber::fmt::layer()
        .json()
        .with_writer(non_blocking);

    // initialize tracing layer
    global::set_text_map_propagator(TraceContextPropagator::new());
    let tracer = opentelemetry_jaeger::new_pipeline()
        .with_service_name("osquery-generator")
        .install_batch(opentelemetry::runtime::Tokio)?;

    // register a subscriber
    let filter = EnvFilter::from_default_env();
    tracing_subscriber::registry()
        .with(filter)
        .with(log_layer)
        .with(tracing_opentelemetry::layer().with_tracer(tracer))
        .init();

    tracing::info!("logger configured successfully");

    // TODO: actually do something here
    Ok(())
}

// TODO: when we have a plugin SDK, hook this binary into it here
#[allow(dead_code)]
async fn event_handler(
    _event: Envelope<RawLog>,
) -> Result<Envelope<GraphDescription>, OsqueryGeneratorError> {
    todo!();
}
