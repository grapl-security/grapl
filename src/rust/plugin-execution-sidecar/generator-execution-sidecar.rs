use clap::Parser;
use plugin_execution_sidecar::{
    config::PluginExecutorConfig,
    plugin_executor::PluginExecutor,
    work::GeneratorWorkProcessor,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (_env, _guard) = grapl_config::init_grapl_env!();
    let plugin_executor_config = PluginExecutorConfig::parse();

    let generator_work_processor = GeneratorWorkProcessor::new().await?;
    let mut plugin_executor =
        PluginExecutor::new(plugin_executor_config, generator_work_processor).await?;
    plugin_executor.main_loop().await
}
