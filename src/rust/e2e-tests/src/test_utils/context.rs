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
    client_factory::services::{
        EventSourceClientConfig,
        GraphSchemaManagerClientConfig,
        PipelineIngressClientConfig,
        PluginRegistryClientConfig,
        ScyllaProvisionerClientConfig,
        UidAllocatorClientConfig,
    },
    graplinc::grapl::api::{
        event_source::v1beta1::{
            client::EventSourceServiceClient,
            CreateEventSourceRequest,
        },
        graph_schema_manager::v1beta1::{
            client::GraphSchemaManagerClient,
            messages::{
                DeploySchemaRequest,
                SchemaType,
            },
        },
        pipeline_ingress::v1beta1::client::PipelineIngressClient,
        plugin_registry::v1beta1::{
            DeployPluginRequest,
            PluginMetadata,
            PluginRegistryServiceClient,
            PluginType,
        },
        scylla_provisioner::v1beta1::{
            client::ScyllaProvisionerClient,
            messages::ProvisionGraphForTenantRequest,
        },
        uid_allocator::v1beta1::{
            client::UidAllocatorServiceClient,
            messages::CreateTenantKeyspaceRequest,
        },
    },
    protocol::service_client::ConnectWithConfig,
};
use test_context::AsyncTestContext;
use uuid::Uuid;

use crate::test_fixtures;

pub struct E2eTestContext {
    pub event_source_client: EventSourceServiceClient,
    pub graph_schema_manager_client: GraphSchemaManagerClient,
    pub plugin_registry_client: PluginRegistryServiceClient,
    pub pipeline_ingress_client: PipelineIngressClient,
    pub plugin_work_queue_psql_client: PsqlQueue,
    pub uid_allocator_client: UidAllocatorServiceClient,
    pub scylla_provisioner_client: ScyllaProvisionerClient,
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

        let graph_schema_manager_client =
            GraphSchemaManagerClient::connect_with_config(GraphSchemaManagerClientConfig::parse())
                .await
                .expect("graph_schema_manager_client");

        let pipeline_ingress_client =
            PipelineIngressClient::connect_with_config(PipelineIngressClientConfig::parse())
                .await
                .expect("pipeline_ingress_client");

        let plugin_work_queue_psql_client =
            PsqlQueue::init_with_config(PluginWorkQueueDbConfig::parse())
                .await
                .expect("plugin_work_queue");

        let uid_allocator_client =
            UidAllocatorServiceClient::connect_with_config(UidAllocatorClientConfig::parse())
                .await
                .expect("uid_allocator_client");

        let scylla_provisioner_client =
            ScyllaProvisionerClient::connect_with_config(ScyllaProvisionerClientConfig::parse())
                .await
                .expect("scylla_provisioner_client");

        Self {
            event_source_client,
            graph_schema_manager_client,
            plugin_registry_client,
            pipeline_ingress_client,
            plugin_work_queue_psql_client,
            uid_allocator_client,
            scylla_provisioner_client,
            _guard,
        }
    }
}

#[derive(Debug)]
pub struct SetupGeneratorResult {
    pub tenant_id: Uuid,
    pub generator_plugin_id: Uuid,
    pub event_source_id: Uuid,
}

pub struct SetupGeneratorOptions {
    pub test_name: String,
    pub tenant_id: Uuid,
    pub generator_artifact: Bytes,
    pub should_deploy_generator: bool,
}

impl E2eTestContext {
    #[tracing::instrument(skip(self), err)]
    pub async fn create_tenant(&mut self) -> eyre::Result<Uuid> {
        tracing::info!("creating tenant");
        let tenant_id = Uuid::new_v4();
        self.uid_allocator_client
            .create_tenant_keyspace(CreateTenantKeyspaceRequest { tenant_id })
            .await?;
        tracing::info!("provisioning graph");
        self.scylla_provisioner_client
            .provision_graph_for_tenant(ProvisionGraphForTenantRequest { tenant_id })
            .await?;
        tracing::info!("created tenant");
        Ok(tenant_id)
    }

    async fn provision_example_graph_schema(&mut self, tenant_id: uuid::Uuid) -> eyre::Result<()> {
        let mut graph_schema_manager_client = self.graph_schema_manager_client.clone();

        fn get_example_graphql_schema() -> Result<Bytes, std::io::Error> {
            // This path is created in rust/Dockerfile
            let path = "/test-fixtures/example_schemas/example.graphql";
            std::fs::read(path).map(Bytes::from)
        }

        graph_schema_manager_client
            .deploy_schema(DeploySchemaRequest {
                tenant_id,
                schema: get_example_graphql_schema().unwrap(),
                schema_type: SchemaType::GraphqlV0,
                schema_version: 0,
            })
            .await?;
        Ok(())
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

        self.provision_example_graph_schema(tenant_id).await?;

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

    pub async fn setup_suspicious_svchost_analyzer(
        &mut self,
        tenant_id: uuid::Uuid,
        test_name: &str,
    ) -> eyre::Result<uuid::Uuid> {
        let analyzer_artifact = test_fixtures::get_suspicious_svchost_analyzer()?;
        let analyzer_plugin_id = self
            .create_analyzer(tenant_id, test_name.to_owned(), analyzer_artifact)
            .await?;
        self.deploy_analyzer(analyzer_plugin_id).await?;
        Ok(analyzer_plugin_id)
    }

    pub async fn setup_sysmon_generator(
        &mut self,
        tenant_id: uuid::Uuid,
        test_name: &str,
    ) -> eyre::Result<SetupGeneratorResult> {
        let generator_artifact = test_fixtures::get_sysmon_generator()?;
        self.setup_generator(SetupGeneratorOptions {
            tenant_id,
            test_name: test_name.to_owned(),
            generator_artifact,
            should_deploy_generator: true,
        })
        .await
    }

    pub async fn setup_generator(
        &mut self,
        options: SetupGeneratorOptions,
    ) -> eyre::Result<SetupGeneratorResult> {
        tracing::info!(">> Generator Setup for {}", options.test_name);

        let tenant_id = options.tenant_id;

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

        let setup_result = SetupGeneratorResult {
            tenant_id,
            generator_plugin_id: generator_id,
            event_source_id,
        };

        tracing::info!(
            message = ">> Generator Setup result",
            result = ?setup_result,
        );

        Ok(setup_result)
    }
}
