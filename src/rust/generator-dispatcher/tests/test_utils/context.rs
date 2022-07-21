use clap::Parser;
use grapl_tracing::{
    setup_tracing,
    WorkerGuard,
};
use plugin_work_queue::{
    psql_queue::PsqlQueue,
    PluginWorkQueueDbConfig,
};
use rust_proto::graplinc::grapl::api::pipeline_ingress::v1beta1::client::PipelineIngressClient;
use rust_proto_clients::{
    get_grpc_client_with_options,
    services::PipelineIngressClientConfig,
    GetGrpcClientOptions,
};
use test_context::AsyncTestContext;

pub struct GeneratorDispatcherTestContext {
    pub pipeline_ingress_client: PipelineIngressClient,
    pub plugin_work_queue_psql_client: PsqlQueue,
    pub _guard: WorkerGuard,
}

const SERVICE_NAME: &'static str = "generator-dispatcher-integration-tests";

#[async_trait::async_trait]
impl AsyncTestContext for GeneratorDispatcherTestContext {
    async fn setup() -> Self {
        let _guard = setup_tracing(SERVICE_NAME).expect("setup_tracing");

        let client_config = PipelineIngressClientConfig::parse();
        let pipeline_ingress_client = get_grpc_client_with_options(
            client_config,
            GetGrpcClientOptions {
                perform_healthcheck: true,
            },
        )
        .await
        .expect("pipeline_ingress_client");

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
