use generator_sdk::server::{
    self,
    GeneratorServiceConfig,
};
use opentelemetry::{
    global,
    sdk::propagation::TraceContextPropagator,
};
use sysmon_generator::api;
use tracing_subscriber::{
    prelude::*,
    EnvFilter,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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

    let config = GeneratorServiceConfig::from_env_vars();
    let generator = api::SysmonGenerator {};
    server::exec_service(generator, config).await
}
