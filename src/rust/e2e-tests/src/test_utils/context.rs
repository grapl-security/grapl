use bytes::Bytes;
use clap::Parser;
use grapl_config::PostgresClient;
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
        event_source::v1beta1::{
            client::EventSourceServiceClient,
            CreateEventSourceRequest,
        },
        pipeline_ingress::v1beta1::client::PipelineIngressClient,
        plugin_registry::v1beta1::{
            DeployPluginRequest,
            PluginMetadata,
            PluginRegistryServiceClient,
            PluginType,
        },
    },
};
use test_context::AsyncTestContext;
use uuid::Uuid;

use crate::test_fixtures;

pub struct E2eTestContext {
    pub event_source_client: EventSourceServiceClient,
    pub plugin_registry_client: PluginRegistryServiceClient,
    pub pipeline_ingress_client: PipelineIngressClient,
    pub plugin_work_queue_psql_client: PsqlQueue,
    pub _guard: WorkerGuard,
}

const SERVICE_NAME: &'static str = "e2e-tests";

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

        let plugin_work_queue_psql_client =
            PsqlQueue::init_with_config(PluginWorkQueueDbConfig::parse())
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

pub struct SetupResult {
    pub tenant_id: Uuid,
    pub plugin_id: Uuid,
    pub event_source_id: Uuid,
}

impl E2eTestContext {
    pub async fn setup_sysmon_generator(
        &mut self,
        test_name: &str,
    ) -> Result<SetupResult, Box<dyn std::error::Error>> {
        let plugin_artifact = test_fixtures::get_sysmon_generator()?;
        self.setup_generator(test_name, plugin_artifact).await
    }

    async fn setup_generator(
        &mut self,
        test_name: &str,
        plugin_artifact: Bytes,
    ) -> Result<SetupResult, Box<dyn std::error::Error>> {
        tracing::info!(">> Settting up Event Source, Plugin");

        let tenant_id = Uuid::new_v4();

        // Register an Event Source
        let event_source = self
            .event_source_client
            .create_event_source(CreateEventSourceRequest {
                display_name: test_name.to_string(),
                description: "arbitrary".to_string(),
                tenant_id,
            })
            .await?;

        // Deploy a Generator Plugin that responds to that event_source_id
        let plugin = {
            let plugin = self
                .plugin_registry_client
                .create_plugin(
                    PluginMetadata {
                        tenant_id,
                        display_name: test_name.to_string(),
                        plugin_type: PluginType::Generator,
                        event_source_id: Some(event_source.event_source_id),
                    },
                    futures::stream::once(async move { plugin_artifact }),
                )
                .await?;

            self.plugin_registry_client
                .deploy_plugin(DeployPluginRequest {
                    plugin_id: plugin.plugin_id,
                })
                .await?;
            plugin
        };

        Ok(SetupResult {
            tenant_id,
            plugin_id: plugin.plugin_id,
            event_source_id: event_source.event_source_id,
        })
    }
}
