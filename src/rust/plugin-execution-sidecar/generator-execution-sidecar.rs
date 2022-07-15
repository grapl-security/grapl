use clap::Parser;
use kafka::config::ProducerConfig;
use plugin_execution_sidecar::{
    config::PluginExecutorConfig,
    plugin_executor::PluginExecutor,
    work::GeneratorWorkProcessor,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (_env, _guard) = grapl_config::init_grapl_env!();
    let plugin_executor_config = PluginExecutorConfig::parse();
    let producer_config = ProducerConfig::parse();

    let generator_work_processor = GeneratorWorkProcessor::new(&plugin_executor_config).await?;
    let mut plugin_executor = PluginExecutor::new(
        plugin_executor_config,
        producer_config,
        generator_work_processor,
    )
    .await?;
    plugin_executor.main_loop().await
}
