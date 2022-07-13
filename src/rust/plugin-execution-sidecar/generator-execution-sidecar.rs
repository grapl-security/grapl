use plugin_execution_sidecar::{
    generator_work_processor::GeneratorWorkProcessor,
    plugin_executor::PluginExecutor,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (_env, _guard) = grapl_config::init_grapl_env!();
    let generator_work_processor = GeneratorWorkProcessor::new().await?;
    let mut plugin_executor = PluginExecutor::new(generator_work_processor).await?;
    plugin_executor.main_loop().await
}
