use opentelemetry::{
    global,
    sdk::propagation::TraceContextPropagator,
    trace::TraceError,
};
pub use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{
    prelude::*,
    EnvFilter,
};

pub fn setup_tracing(service_name: &str) -> Result<WorkerGuard, TraceError> {
    let (non_blocking, guard) = tracing_appender::non_blocking(std::io::stdout());

    // initialize json logging layer
    let log_layer = tracing_subscriber::fmt::layer()
        .json()
        .with_writer(non_blocking);

    // initialize tracing layer
    global::set_text_map_propagator(TraceContextPropagator::new());
    let tracer = opentelemetry_jaeger::new_pipeline()
        .with_service_name(service_name)
        .install_batch(opentelemetry::runtime::Tokio)?;

    // register a subscriber
    let filter = EnvFilter::from_default_env();
    tracing_subscriber::registry()
        .with(filter)
        .with(log_layer)
        .with(tracing_opentelemetry::layer().with_tracer(tracer))
        .init();

    tracing::info!("logger configured successfully");

    Ok(guard)
}
