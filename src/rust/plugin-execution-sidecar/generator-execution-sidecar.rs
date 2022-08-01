use clap::Parser;
use grapl_tracing::setup_tracing;
use plugin_execution_sidecar::{
    config::PluginExecutorConfig,
    plugin_executor::PluginExecutor,
    work::GeneratorWorkProcessor,
};
const SERVICE_NAME: &'static str = "generator-execution-sidecar";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = setup_tracing(SERVICE_NAME)?;
    let plugin_executor_config = PluginExecutorConfig::parse();

    let generator_work_processor = GeneratorWorkProcessor::new(&plugin_executor_config).await?;
    let mut plugin_executor =
        PluginExecutor::new(plugin_executor_config, generator_work_processor).await?;
    plugin_executor.main_loop().await
}
