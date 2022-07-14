use clap::StructOpt;
use generator_dispatcher::{
    config::GeneratorDispatcherConfig,
    GeneratorDispatcher,
};
use plugin_work_queue::client::{
    FromEnv as PWQFromEnv,
    PluginWorkQueueServiceClient,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (_env, _guard) = grapl_config::init_grapl_env!();
    let config = GeneratorDispatcherConfig::parse();
    let plugin_work_queue_client = PluginWorkQueueServiceClient::from_env().await?;
    let worker_pool_size = config.worker_pool_size;
    let mut generator_dispatcher =
        GeneratorDispatcher::new(config, plugin_work_queue_client).await?;

    generator_dispatcher.run(worker_pool_size).await?;

    Ok(())
}
