use clap::Parser;
use plugin_work_queue::{
    server::exec_service,
    PluginWorkQueueServiceConfig,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (_env, _guard) = grapl_config::init_grapl_env!();
    let config = PluginWorkQueueServiceConfig::parse();
    exec_service(config).await?;
    Ok(())
}
