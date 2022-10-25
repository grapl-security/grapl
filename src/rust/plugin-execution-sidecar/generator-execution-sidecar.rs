use std::time::Duration;

use figment::{
    providers::Env,
    Figment,
};
use grapl_tracing::setup_tracing;
use plugin_execution_sidecar::{
    plugin_executor::PluginExecutor,
    work::GeneratorWorkProcessor,
};
use serde::{
    Deserialize,
    Serialize,
};
use uuid::Uuid;

const SERVICE_NAME: &'static str = "generator-execution-sidecar";

#[derive(Clone, Debug, Deserialize, Serialize)]
struct SidecarConfig {
    plugin_id: Uuid,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = setup_tracing(SERVICE_NAME)?;
    tracing::info!("logging configured successfully");

    // Give the plugin a little time to become available.
    tokio::time::sleep(Duration::from_secs(5)).await;

    let generator_work_processor = GeneratorWorkProcessor::new().await?;

    let sidecar_config: SidecarConfig = Figment::new()
        .merge(Env::prefixed("GENERATOR_EXECUTION_SIDECAR_"))
        .extract()?;

    let mut plugin_executor =
        PluginExecutor::new(sidecar_config.plugin_id, generator_work_processor).await?;

    tracing::info!("starting generator executor");

    plugin_executor.main_loop().await
}
