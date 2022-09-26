use opentelemetry::{
    global,
    sdk::propagation::TraceContextPropagator,
};
pub use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{
    prelude::*,
    EnvFilter,
};

#[derive(thiserror::Error, Debug)]
pub enum SetupTracingError {
    #[error("TraceError")]
    TraceError(#[from] opentelemetry::trace::TraceError),
}

fn otel_jaeger_enabled() -> bool {
    // If this environment variable isn't set, don't bother hooking up Jaeger
    std::env::var("OTEL_EXPORTER_JAEGER_AGENT_HOST").is_ok()
}

pub fn setup_tracing(service_name: &str) -> Result<WorkerGuard, SetupTracingError> {
    let (non_blocking, guard) = tracing_appender::non_blocking(std::io::stdout());

    // initialize json logging layer
    let log_layer = tracing_subscriber::fmt::layer()
        .with_file(true)
        .with_line_number(true)
        .json()
        .with_writer(non_blocking);

    // initialize tracing layer
    global::set_text_map_propagator(TraceContextPropagator::new());
    let jaeger_tracer = opentelemetry_jaeger::new_pipeline()
        .with_service_name(service_name)
        .install_batch(opentelemetry::runtime::Tokio)?;

    // register a subscriber
    let filter = EnvFilter::from_default_env();
    {
        // Builder pattern is difficult here
        let registry = tracing_subscriber::registry().with(filter).with(log_layer);
        if otel_jaeger_enabled() {
            registry
                .with(tracing_opentelemetry::layer().with_tracer(jaeger_tracer))
                .init();
        } else {
            registry.init();
            tracing::warn!("Skipping Jaeger tracer");
        }
    };

    tracing::info!("Tracer configured successfully");

    Ok(guard)
}
