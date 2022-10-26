use figment::{
    providers::Env,
    Figment,
};
use grapl_tracing::setup_tracing;
use plugin_execution_sidecar::{
    plugin_executor::{
        PluginExecutor,
        SidecarConfig,
    },
    work::AnalyzerWorkProcessor,
};

const SERVICE_NAME: &'static str = "analyzer-execution-sidecar";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = setup_tracing(SERVICE_NAME)?;

    tracing::info!("logging configured successfully");

    // Give the plugin a little time to become available.
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    let sidecar_config: SidecarConfig = Figment::new()
        .merge(Env::prefixed("ANALYZER_EXECUTION_SIDECAR_"))
        .extract()?;

    let analyzer_work_processor = AnalyzerWorkProcessor::new().await?;
    let mut plugin_executor =
        PluginExecutor::new(sidecar_config.plugin_id(), analyzer_work_processor).await?;

    tracing::info!("starting analyzer executor");

    plugin_executor.main_loop().await
}
