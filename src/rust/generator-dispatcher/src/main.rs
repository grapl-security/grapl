use clap::StructOpt;
use futures::StreamExt;
use generator_dispatcher::config::GeneratorDispatcherConfig;
use kafka::Consumer;
use plugin_work_queue::client::{
    FromEnv,
    PluginWorkQueueServiceClient,
};
use rust_proto_new::graplinc::grapl::{
    api::plugin_work_queue::v1beta1::{
        ExecutionJob,
        PutExecuteGeneratorRequest,
    },
    pipeline::{
        v1beta1::RawLog,
        v1beta2::Envelope,
    },
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (_env, _guard) = grapl_config::init_grapl_env!();
    let config = GeneratorDispatcherConfig::parse();
    let mut generator_dispatcher = GeneratorDispatcher::new(config).await?;
    generator_dispatcher.main_loop().await
}

struct GeneratorDispatcher {
    plugin_work_queue_client: PluginWorkQueueServiceClient,
    raw_logs_consumer: Consumer<Envelope<RawLog>>,
}

impl GeneratorDispatcher {
    #[tracing::instrument(err)]
    async fn new(config: GeneratorDispatcherConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let plugin_work_queue_client = PluginWorkQueueServiceClient::from_env().await?;

        let raw_logs_consumer: Consumer<Envelope<RawLog>> = Consumer::new(config.kafka_config)?;

        Ok(Self {
            plugin_work_queue_client,
            raw_logs_consumer,
        })
    }

    #[tracing::instrument(skip(self), err)]
    async fn main_loop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut stream = self.raw_logs_consumer.stream()?;
        while let Some(raw_log_result) = stream.next().await {
            let Envelope {
                inner_message,
                metadata,
            } = raw_log_result?;

            // HAX TODO we need event source management
            let hax_temp_plugin_id = uuid::Uuid::new_v4();
            let execution_job = ExecutionJob {
                data: inner_message.log_event.to_vec(),
                plugin_id: hax_temp_plugin_id,
                tenant_id: metadata.tenant_id,
            };
            self.plugin_work_queue_client
                .put_execute_generator(PutExecuteGeneratorRequest { execution_job })
                .await?;
        }
        // Should we let the process exit if that while-let fails?
        Ok(())
    }
}
