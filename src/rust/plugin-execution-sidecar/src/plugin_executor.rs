use std::time::Duration;

use clap::Parser;
use rust_proto::graplinc::grapl::api::plugin_work_queue::v1beta1::PluginWorkQueueServiceClient;
use rust_proto_clients::{
    get_grpc_client,
    services::PluginWorkQueueClientConfig,
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
        let client_config = PluginWorkQueueClientConfig::parse();
        let plugin_work_queue_client = get_grpc_client(client_config).await?;

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
                let tenant_id = job.tenant_id();
                let trace_id = job.trace_id();
                let event_source_id = job.event_source_id();
                // Process the job
                let process_result = self
                    .plugin_work_processor
                    .process_job(&self.config, job)
                    .await;

                if let Err(e) = process_result.as_ref() {
                    tracing::error!(
                        message = "Error processing job",
                        request_id = ?request_id,
                        error = ?e,
                        tenant_id = %tenant_id,
                        trace_id = %trace_id,
                        event_source_id = %event_source_id,
                    );
                }

                // Inform plugin-work-queue whether it worked or if we need
                // to retry
                self.plugin_work_processor
                    .ack_work(
                        &self.config,
                        &mut self.plugin_work_queue_client,
                        process_result,
                        request_id,
                        tenant_id,
                        trace_id,
                        event_source_id,
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
