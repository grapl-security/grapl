use std::time::Duration;

use plugin_executor::generator_client::FromEnv as generator_from_env;
use plugin_work_queue::client::{
    FromEnv as pwq_from_env,
    PluginWorkQueueServiceClient,
};
use rust_proto::graplinc::grapl::api::{
    plugin_sdk::generators::v1beta1::{
        client::GeneratorServiceClient,
        RunGeneratorRequest,
    },
    plugin_work_queue::v1beta1::{
        AcknowledgeGeneratorRequest,
        ExecutionJob,
        GetExecuteGeneratorRequest,
    },
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (_env, _guard) = grapl_config::init_grapl_env!();
    let mut generator_executor = GeneratorExecutor::new().await?;
    generator_executor.main_loop().await
}

struct GeneratorExecutor {
    generator_service_client: GeneratorServiceClient,
    plugin_work_queue_client: PluginWorkQueueServiceClient,
    plugin_id: uuid::Uuid,
}

impl GeneratorExecutor {
    #[tracing::instrument(err)]
    async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let generator_service_client = GeneratorServiceClient::from_env().await?;
        let plugin_work_queue_client = PluginWorkQueueServiceClient::from_env().await?;

        // TODO: perhaps change to Clap
        let plugin_id = std::env::var("PLUGIN_ID").expect("PLUGIN_ID");
        let plugin_id = uuid::Uuid::parse_str(&plugin_id)?;

        Ok(Self {
            generator_service_client,
            plugin_work_queue_client,
            plugin_id,
        })
    }

    #[tracing::instrument(skip(self), err)]
    async fn main_loop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Continually scan for new work for this Plugin.
        let get_request = GetExecuteGeneratorRequest {
            plugin_id: self.plugin_id,
        };
        while let Ok(get_execute_response) = self
            .plugin_work_queue_client
            .get_execute_generator(get_request.clone())
            .await
        {
            let request_id = get_execute_response.request_id;
            if let Some(job) = get_execute_response.execution_job {
                // Process the job
                let process_result = self.process_job(job).await;

                // Inform plugin-work-queue whether it worked or if we need
                // to retry
                self.plugin_work_queue_client
                    .acknowledge_generator(AcknowledgeGeneratorRequest {
                        request_id,
                        plugin_id: self.plugin_id,
                        success: process_result.is_ok(),
                    })
                    .await?;
            } else {
                tokio::time::sleep(Duration::from_secs(5)).await;
                continue;
            }
        }
        // Should we let the process exit if that while-let fails?
        Ok(())
    }

    #[tracing::instrument(skip(self, job), err)]
    async fn process_job(&mut self, job: ExecutionJob) -> Result<(), Box<dyn std::error::Error>> {
        let _run_generator_response = self
            .generator_service_client
            .run_generator(RunGeneratorRequest { data: job.data })
            .await?;

        //kafka_stream.put(generated_graphs).await.unwrap();
        Ok(()) // TODO replace with above
    }
}
