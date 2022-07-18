use std::time::Duration;

use clap::Parser;
use grapl_tracing::{setup_tracing, WorkerGuard};
use plugin_work_queue::{
    psql_queue::PsqlQueue,
    PluginWorkQueueDbConfig,
};
use rust_proto::{
    graplinc::grapl::api::pipeline_ingress::v1beta1::client::PipelineIngressClient,
    protocol::{
        healthcheck::client::HealthcheckClient,
        service_client::NamedService,
    },
};
use test_context::AsyncTestContext;

pub struct GeneratorDispatcherTestContext {
    pub pipeline_ingress_client: PipelineIngressClient,
    pub plugin_work_queue_psql_client: PsqlQueue,
    pub _guard: WorkerGuard,
}

#[async_trait::async_trait]
impl AsyncTestContext for GeneratorDispatcherTestContext {
    async fn setup() -> Self {
        let _guard = setup_tracing("generator-dispatcher-integration-tests").unwrap();

        let endpoint = std::env::var("PIPELINE_INGRESS_CLIENT_ADDRESS")
            .expect("missing environment variable PIPELINE_INGRESS_CLIENT_ADDRESS");

        tracing::info!(
            message = "waiting 10s for pipeline-ingress to report healthy",
            endpoint = %endpoint,
        );

        HealthcheckClient::wait_until_healthy(
            endpoint.clone(),
            PipelineIngressClient::SERVICE_NAME,
            Duration::from_millis(10000),
            Duration::from_millis(500),
        )
        .await
        .expect("pipeline-ingress never reported healthy");

        tracing::info!("connecting pipeline-ingress gRPC client");
        let pipeline_ingress_client = PipelineIngressClient::connect(endpoint.clone())
            .await
            .expect("could not configure gRPC client");

        let plugin_work_queue_psql_client = PsqlQueue::try_from(PluginWorkQueueDbConfig::parse())
            .await
            .expect("plugin_work_queue");

        GeneratorDispatcherTestContext {
            pipeline_ingress_client,
            plugin_work_queue_psql_client,
            _guard,
        }
    }
}
