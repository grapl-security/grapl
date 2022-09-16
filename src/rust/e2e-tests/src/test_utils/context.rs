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
use rust_proto::graplinc::grapl::api::{
    client_factory::services::{
        EventSourceClientConfig,
        PipelineIngressClientConfig,
        PluginRegistryClientConfig,
    },
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
    protocol::service_client::ConnectWithConfig,
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

const SERVICE_NAME: &'static str = "E2eTestContext";

#[async_trait::async_trait]
impl AsyncTestContext for E2eTestContext {
    async fn setup() -> Self {
        let _guard = setup_tracing(SERVICE_NAME).expect("setup_tracing");

        let event_source_client =
            EventSourceServiceClient::connect_with_config(EventSourceClientConfig::parse())
                .await
                .expect("event_source_client");

        let plugin_registry_client =
            PluginRegistryServiceClient::connect_with_config(PluginRegistryClientConfig::parse())
                .await
                .expect("plugin_registry_client");

        let pipeline_ingress_client =
            PipelineIngressClient::connect_with_config(PipelineIngressClientConfig::parse())
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

#[derive(Debug)]
pub struct SetupResult {
    pub tenant_id: Uuid,
    pub plugin_id: Uuid,
    pub event_source_id: Uuid,
}

pub struct SetupGeneratorOptions {
    pub test_name: String,
    pub generator_artifact: Bytes,
    pub should_deploy_generator: bool,
}

impl E2eTestContext {
    pub async fn create_tenant(&mut self) -> eyre::Result<Uuid> {
        tracing::info!("creating tenant");
        // TODO: actually create a real tenant
        Ok(Uuid::new_v4())
    }

    pub async fn create_event_source(
        &mut self,
        tenant_id: Uuid,
        display_name: String,
        description: String,
    ) -> eyre::Result<Uuid> {
        let event_source = self
            .event_source_client
            .create_event_source(CreateEventSourceRequest {
                display_name,
                description,
                tenant_id,
            })
            .await?;

        Ok(event_source.event_source_id)
    }

    pub async fn create_generator(
        &mut self,
        tenant_id: Uuid,
        display_name: String,
        event_source_id: Uuid,
        generator_artifact: Bytes,
    ) -> eyre::Result<Uuid> {
        let generator = self
            .plugin_registry_client
            .create_plugin(
                PluginMetadata::new(
                    tenant_id,
                    display_name,
                    PluginType::Generator,
                    Some(event_source_id),
                ),
                futures::stream::once(async move { generator_artifact }),
            )
            .await?;

        Ok(generator.plugin_id())
    }

    pub async fn deploy_generator(&mut self, generator_id: Uuid) -> eyre::Result<()> {
        self.plugin_registry_client
            .deploy_plugin(DeployPluginRequest::new(generator_id))
            .await?;

        Ok(())
    }

    pub async fn create_analyzer(
        &mut self,
        tenant_id: Uuid,
        display_name: String,
        analyzer_artifact: Bytes,
    ) -> eyre::Result<Uuid> {
        tracing::info!(
            message = "Creating analyzer plugin",
            tenant_id =% tenant_id,
        );

        let analyzer = self
            .plugin_registry_client
            .create_plugin(
                PluginMetadata::new(tenant_id, display_name, PluginType::Analyzer, None),
                futures::stream::once(async move { analyzer_artifact }),
            )
            .await?;

        Ok(analyzer.plugin_id())
    }

    pub async fn deploy_analyzer(&mut self, analyzer_id: Uuid) -> eyre::Result<()> {
        tracing::info!(
            message = "deploying analyzer",
            analyzer_id =% analyzer_id,
        );

        self.plugin_registry_client
            .deploy_plugin(DeployPluginRequest::new(analyzer_id))
            .await?;

        Ok(())
    }

    pub async fn setup_sysmon_generator(&mut self, test_name: &str) -> eyre::Result<SetupResult> {
        let generator_artifact = test_fixtures::get_sysmon_generator()?;
        self.setup_generator(SetupGeneratorOptions {
            test_name: test_name.to_owned(),
            generator_artifact,
            should_deploy_generator: true,
        })
        .await
    }

    pub async fn setup_generator(
        &mut self,
        options: SetupGeneratorOptions,
    ) -> eyre::Result<SetupResult> {
        tracing::info!(">> Generator Setup for {}", options.test_name);

        let tenant_id = self.create_tenant().await?;

        // Register an Event Source
        let event_source_id = self
            .create_event_source(
                tenant_id,
                options.test_name.clone(),
                "arbitrary".to_string(),
            )
            .await?;

        // Deploy a Generator Plugin that responds to that event_source_id
        let generator_id = {
            let generator_id = self
                .create_generator(
                    tenant_id,
                    options.test_name.clone(),
                    event_source_id,
                    options.generator_artifact,
                )
                .await?;

            if options.should_deploy_generator {
                self.deploy_generator(generator_id).await?;
            }

            generator_id
        };

        let setup_result = SetupResult {
            tenant_id,
            plugin_id: generator_id,
            event_source_id,
        };

        tracing::info!(
            message = ">> Generator Setup result",
            result = ?setup_result,
        );

        Ok(setup_result)
    }
}
