use std::time::Duration;

use plugin_work_queue::client::{
    FromEnv as pwq_from_env,
    PluginWorkQueueServiceClient,
};

use crate::work::{
    PluginWorkProcessor,
    Workload,
};

pub struct PluginExecutor<P: PluginWorkProcessor> {
    plugin_work_processor: P,
    plugin_work_queue_client: PluginWorkQueueServiceClient,
    plugin_id: uuid::Uuid,
}

impl<P> PluginExecutor<P>
where
    P: PluginWorkProcessor,
{
    pub async fn new(plugin_work_processor: P) -> Result<Self, Box<dyn std::error::Error>> {
        let plugin_work_queue_client = PluginWorkQueueServiceClient::from_env().await?;

        // TODO: perhaps change to Clap
        let plugin_id = std::env::var("PLUGIN_ID").expect("PLUGIN_ID");
        let plugin_id = uuid::Uuid::parse_str(&plugin_id)?;

        Ok(Self {
            plugin_work_processor,
            plugin_work_queue_client,
            plugin_id,
        })
    }

    #[tracing::instrument(skip(self), err)]
    pub async fn main_loop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Continually scan for new work for this Plugin.
        while let Ok(work) = self
            .plugin_work_processor
            .get_work(&mut self.plugin_work_queue_client, self.plugin_id)
            .await
        {
            let request_id = work.request_id();
            if let Some(job) = work.maybe_job() {
                // Process the job
                let process_result = self.plugin_work_processor.process_job(job).await;
                let success = process_result.is_ok();

                // Inform plugin-work-queue whether it worked or if we need
                // to retry
                self.plugin_work_processor
                    .ack_work(
                        &mut self.plugin_work_queue_client,
                        self.plugin_id,
                        request_id,
                        success,
                    )
                    .await?;
            } else {
                tokio::time::sleep(Duration::from_secs(5)).await;
                continue;
            }
        }
        // Should we let the process exit if that while-let fails?
        Ok(())
    }
}
