use std::time::Duration;

use generator_sdk::client::GeneratorClient;
use plugin_executor::upstreams::plugin_work_queue_client_from_env;
use plugin_work_queue::client::PluginWorkQueueServiceClient;
use rust_proto::plugin_work_queue::{
    ExecutionJob,
    GetExecuteGeneratorRequest,
};
use structopt::StructOpt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    GeneratorExecutor::new().await?.main_loop().await
}

struct GeneratorExecutor {
    generator_client: GeneratorClient,
    plugin_work_queue_client: PluginWorkQueueServiceClient,
}

impl GeneratorExecutor {
    async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let generator_client = {
            let generator_client_config = generator_sdk::client_config::ClientConfig::from_args();
            GeneratorClient::from(generator_client_config)
        };

        let plugin_work_queue_client = plugin_work_queue_client_from_env().await?;

        Ok(Self {
            generator_client,
            plugin_work_queue_client,
        })
    }

    async fn main_loop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        while let Ok(get_execute_response) = self
            .plugin_work_queue_client
            .get_execute_generator(GetExecuteGeneratorRequest {})
            .await
        {
            if let Some(job) = get_execute_response.execution_job {
                self.process_job(job, get_execute_response.request_id)
                    .await?
            } else {
                tokio::time::sleep(Duration::from_secs(5)).await;
                continue;
            }
        }
        // Should we let the process exit if that while-let fails?
        Ok(())
    }

    #[tracing::instrument(skip(self, job), err)]
    async fn process_job(
        &mut self,
        job: ExecutionJob,
        request_id: i64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let _generated_graphs = self
            .generator_client
            .run_generator(job.data, job.plugin_id.to_string())
            .await?;

        //kafka_stream.put(generated_graphs).await.unwrap();

        //plugin_work_queue_client.acknowledge().await?.unwrap();
        Ok(())
    }
}
