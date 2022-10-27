use figment::{
    providers::Env,
    Figment,
};
use generator_dispatcher::{
    config::GeneratorDispatcherConfig,
    GeneratorDispatcher,
};
use rust_proto::graplinc::grapl::api::{
    client::Connect,
    plugin_work_queue::v1beta1::PluginWorkQueueClient,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = grapl_tracing::setup_tracing("generator-dispatcher")?;
    let config = GeneratorDispatcherConfig::parse();
    let plugin_work_queue_client_config = Figment::new()
        .merge(Env::prefixed("PLUGIN_WORK_QUEUE_"))
        .extract()?;
    let plugin_work_queue_client =
        PluginWorkQueueClient::connect(plugin_work_queue_client_config).await?;
    let worker_pool_size = config.params.worker_pool_size;
    let mut generator_dispatcher =
        GeneratorDispatcher::new(config, plugin_work_queue_client).await?;

    generator_dispatcher.run(worker_pool_size).await?;

    Ok(())
}
