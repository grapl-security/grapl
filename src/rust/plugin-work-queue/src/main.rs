use clap::Parser;
use kafka::config::ProducerConfig;
use plugin_work_queue::{
    server::exec_service,
    ConfigUnion,
    PluginWorkQueueDbConfig,
    PluginWorkQueueServiceConfig,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (_env, _guard) = grapl_config::init_grapl_env!();
    let service_config = PluginWorkQueueServiceConfig::parse();
    let db_config = PluginWorkQueueDbConfig::parse();
    let generator_producer_config =
        ProducerConfig::with_topic_env_var("KAFKA_PRODUCER_TOPIC_GENERATOR");
    // TODO let analyzer_producer_config = ...
    exec_service(ConfigUnion {
        service_config,
        db_config,
        generator_producer_config,
    })
    .await?;
    Ok(())
}
