use generator_sdk::server::{
    self,
    GeneratorServiceConfig,
};
use opentelemetry::{
    global,
    sdk::propagation::TraceContextPropagator,
};
use rust_proto::graplinc::grapl::api::plugin_sdk::generators::v1beta1::{
    server::GeneratorApi,
    GeneratedGraph,
    RunGeneratorRequest,
    RunGeneratorResponse,
};
use sysmon_parser::SysmonEvent;
use tracing_subscriber::{
    prelude::*,
    EnvFilter,
};

mod error;
mod models;

use crate::error::SysmonGeneratorError;

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
    let generator = SysmonGenerator {};
    server::exec_service(generator, config).await
}

pub struct SysmonGenerator {}

#[async_trait::async_trait]
impl GeneratorApi for SysmonGenerator {
    type Error = SysmonGeneratorError;

    #[tracing::instrument(skip(self, request), err)]
    async fn run_generator(
        &self,
        request: RunGeneratorRequest,
    ) -> Result<RunGeneratorResponse, Self::Error> {
        let sysmon_event = SysmonEvent::from_str(std::str::from_utf8(&request.data)?)?;

        match models::generate_graph_from_event(&sysmon_event)? {
            Some(graph_description) => Ok(RunGeneratorResponse {
                generated_graph: GeneratedGraph { graph_description },
            }),
            None => {
                // We do not expect to handle all Sysmon event types.
                // So we'd just return an empty Graph Description.
                Ok(RunGeneratorResponse::default())
            }
        }
    }
}
