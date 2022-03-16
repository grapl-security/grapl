use graph_generator_lib::run_graph_generator;
use sysmon_generator_lib::{
    generator::SysmonGenerator,
    metrics::SysmonGeneratorMetrics,
    serialization::SysmonDecoder,
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
        move |cache| SysmonGenerator::new(cache, SysmonGeneratorMetrics::new(&service_name)),
        SysmonDecoder::default(),
    )
    .await;

    Ok(())
}
