use graph_generator_lib::*;
use osquery_generator_lib::{
    generator::OSQueryGenerator,
    metrics::OSQueryGeneratorMetrics,
};
#[tokio::main]
#[tracing::instrument]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (env, _guard) = grapl_config::init_grapl_env!();
    let service_name = env.service_name.clone();

    tracing::info!(
        message = "Starting generator.",
        name =% service_name
    );

    run_graph_generator(
        env,
        move |cache| OSQueryGenerator::new(cache, OSQueryGeneratorMetrics::new(&service_name)),
        grapl_service::decoder::NdjsonDecoder::default(),
    )
    .await;

    Ok(())
}
