use std::time::Duration;

use plugin_work_queue::client::{
    FromEnv as pwq_from_env,
    PluginWorkQueueServiceClient,
};

use crate::{
    config::PluginExecutorConfig,
    work::{
        PluginWorkProcessor,
        Workload,
    },
};

pub struct PluginExecutor<P: PluginWorkProcessor> {
    plugin_work_processor: P,
    plugin_work_queue_client: PluginWorkQueueServiceClient,
    config: PluginExecutorConfig,
}

impl<P> PluginExecutor<P>
where
    P: PluginWorkProcessor,
{
    pub async fn new(
        config: PluginExecutorConfig,
        plugin_work_processor: P,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let plugin_work_queue_client = PluginWorkQueueServiceClient::from_env().await?;

        Ok(Self {
            plugin_work_processor,
            plugin_work_queue_client,
            config,
        })
    }

    #[tracing::instrument(skip(self), err)]
    pub async fn main_loop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Continually scan for new work for this Plugin.
        while let Ok(work) = self
            .plugin_work_processor
            .get_work(&self.config, &mut self.plugin_work_queue_client)
            .await
        {
            let request_id = work.request_id();
            if let Some(job) = work.maybe_job() {
                // Process the job
                let process_result = self
                    .plugin_work_processor
                    .process_job(&self.config, job)
                    .await;
                let success = process_result.is_ok();

                // Inform plugin-work-queue whether it worked or if we need
                // to retry
                self.plugin_work_processor
                    .ack_work(
                        &self.config,
                        &mut self.plugin_work_queue_client,
                        request_id,
                        success,
                    )
                    .await?;
            } else {
                tokio::time::sleep(Duration::from_secs(1)).await;
                continue;
            }
        }
        Err("Unable to get new work".into())
    }
}
