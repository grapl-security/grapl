use clap::Parser;
use grapl_tracing::setup_tracing;
use plugin_execution_sidecar::{
    config::PluginExecutorConfig,
    plugin_executor::PluginExecutor,
    work::AnalyzerWorkProcessor,
};

const SERVICE_NAME: &'static str = "analyzer-execution-sidecar";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = setup_tracing(SERVICE_NAME)?;
    let plugin_executor_config = PluginExecutorConfig::parse();

    tracing::info!("logging configured successfully");

    // Give the plugin a little time to become available.
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    let analyzer_work_processor = AnalyzerWorkProcessor::new(&plugin_executor_config).await?;
    let mut plugin_executor =
        PluginExecutor::new(plugin_executor_config, analyzer_work_processor).await?;

    tracing::info!("starting analyzer executor");

    plugin_executor.main_loop().await
}
