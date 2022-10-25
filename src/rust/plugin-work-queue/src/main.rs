use clap::Parser;
use grapl_tracing::setup_tracing;
use kafka::config::ProducerConfig;
use plugin_work_queue::{
    server::exec_service,
    ConfigUnion,
    PluginWorkQueueDbConfig,
    PluginWorkQueueServiceConfig,
};
const SERVICE_NAME: &'static str = "plugin-work-queue";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = setup_tracing(SERVICE_NAME)?;
    let service_config = PluginWorkQueueServiceConfig::parse();
    let db_config = PluginWorkQueueDbConfig::parse();
    let generator_producer_config =
        ProducerConfig::with_topic_env_var("GENERATOR_KAFKA_PRODUCER_TOPIC");
    // TODO let analyzer_producer_config = ...
    exec_service(ConfigUnion {
        service_config,
        db_config,
        generator_producer_config,
    })
    .await?;
    Ok(())
}
