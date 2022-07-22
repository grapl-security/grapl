use clap::Parser;
use grapl_tracing::{
    setup_tracing,
    WorkerGuard,
};
use plugin_work_queue::{
    psql_queue::PsqlQueue,
    PluginWorkQueueDbConfig,
};
use rust_proto::{
    client_factory::{
        build_grpc_client_with_options,
        services::{
            EventSourceClientConfig,
            PipelineIngressClientConfig,
            PluginRegistryClientConfig,
        },
        BuildGrpcClientOptions,
    },
    graplinc::grapl::api::{
        event_source::v1beta1::client::EventSourceServiceClient,
        pipeline_ingress::v1beta1::client::PipelineIngressClient,
        plugin_registry::v1beta1::PluginRegistryServiceClient,
    },
};
use test_context::AsyncTestContext;

pub struct E2eTestContext {
    pub event_source_client: EventSourceServiceClient,
    pub plugin_registry_client: PluginRegistryServiceClient,
    pub pipeline_ingress_client: PipelineIngressClient,
    pub plugin_work_queue_psql_client: PsqlQueue,
    pub _guard: WorkerGuard,
}

const SERVICE_NAME: &'static str = "generator-dispatcher-integration-tests";

#[async_trait::async_trait]
impl AsyncTestContext for E2eTestContext {
    async fn setup() -> Self {
        let _guard = setup_tracing(SERVICE_NAME).expect("setup_tracing");
        let get_grpc_options = BuildGrpcClientOptions {
            perform_healthcheck: true,
            ..Default::default()
        };

        let event_source_client = build_grpc_client_with_options(
            EventSourceClientConfig::parse(),
            get_grpc_options.clone(),
        )
        .await
        .expect("event_source_client");

        let plugin_registry_client = build_grpc_client_with_options(
            PluginRegistryClientConfig::parse(),
            get_grpc_options.clone(),
        )
        .await
        .expect("event_source_client");

        let pipeline_ingress_client = build_grpc_client_with_options(
            PipelineIngressClientConfig::parse(),
            get_grpc_options.clone(),
        )
        .await
        .expect("pipeline_ingress_client");

        let plugin_work_queue_psql_client = PsqlQueue::try_from(PluginWorkQueueDbConfig::parse())
            .await
            .expect("plugin_work_queue");

        Self {
            event_source_client,
            plugin_registry_client,
            pipeline_ingress_client,
            plugin_work_queue_psql_client,
            _guard,
        }
    }
}
