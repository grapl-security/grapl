use std::{
    net::SocketAddr,
    time::Duration,
};

use futures::StreamExt;
use grapl_config::env_helpers::FromEnv;
use rusoto_s3::S3Client;
use rust_proto::{
    graplinc::grapl::api::plugin_registry::v1beta1::{
        CreatePluginRequest,
        CreatePluginResponse,
        DeployPluginRequest,
        DeployPluginResponse,
        GetAnalyzersForTenantRequest,
        GetAnalyzersForTenantResponse,
        GetGeneratorsForEventSourceRequest,
        GetGeneratorsForEventSourceResponse,
        GetPluginHealthRequest,
        GetPluginHealthResponse,
        GetPluginRequest,
        GetPluginResponse,
        PluginMetadata,
        PluginRegistryApi,
        PluginRegistryServer,
        PluginType,
        TearDownPluginRequest,
        TearDownPluginResponse,
    },
    protocol::healthcheck::HealthcheckStatus,
};
use tokio::net::TcpListener;
use tonic::async_trait;

use super::{
    create_plugin::upload_stream_multipart_to_s3,
    get_plugin_health,
};
use crate::{
    db::{
        client::{
            DbCreatePluginArgs,
            PluginRegistryDbClient,
        },
        models::PluginRow,
        serde::try_from,
    },
    error::PluginRegistryServiceError,
    nomad::{
        cli::NomadCli,
        client::NomadClient,
    },
    server::{
        create_plugin,
        deploy_plugin,
    },
};

#[derive(clap::Parser, Debug)]
pub struct PluginRegistryConfig {
    #[structopt(flatten)]
    db_config: PluginRegistryDbConfig,
    #[structopt(flatten)]
    service_config: PluginRegistryServiceConfig,
}

#[derive(clap::Parser, Debug)]
pub struct PluginRegistryDbConfig {
    #[clap(long, env)]
    plugin_registry_db_hostname: String,
    #[clap(long, env)]
    plugin_registry_db_port: u16,
    #[clap(long, env)]
    plugin_registry_db_username: String,
    #[clap(long, env)]
    plugin_registry_db_password: String,
}

#[derive(clap::Parser, Debug)]
pub struct PluginRegistryServiceConfig {
    #[clap(long, env = "PLUGIN_REGISTRY_BUCKET_AWS_ACCOUNT_ID")]
    pub bucket_aws_account_id: String,
    #[clap(long, env = "PLUGIN_REGISTRY_BUCKET_NAME")]
    pub bucket_name: String,
    #[clap(long, env)]
    pub plugin_registry_bind_address: SocketAddr,
    #[clap(long, env)]
    pub plugin_bootstrap_container_image: String,
    #[clap(long, env)]
    pub plugin_execution_image: String,
    #[clap(long, env = "PLUGIN_REGISTRY_KERNEL_ARTIFACT_URL")]
    pub kernel_artifact_url: String,
    #[clap(long, env = "PLUGIN_REGISTRY_ROOTFS_ARTIFACT_URL")]
    pub rootfs_artifact_url: String,
    #[clap(long, env = "PLUGIN_REGISTRY_HAX_DOCKER_PLUGIN_RUNTIME_IMAGE")]
    pub hax_docker_plugin_runtime_image: String,
    #[clap(
        long,
        env = "PLUGIN_REGISTRY_ARTIFACT_SIZE_LIMIT_MB",
        default_value = "250"
    )]
    pub artifact_size_limit_mb: usize,
    // --- Pass through a couple env vars also used for this binary
    #[clap(long, env)]
    pub rust_log: String,
    #[clap(long, env)]
    pub otel_exporter_jaeger_agent_host: String,
    #[clap(long, env)]
    pub otel_exporter_jaeger_agent_port: String,
}

pub struct PluginRegistry {
    db_client: PluginRegistryDbClient,
    nomad_client: NomadClient,
    nomad_cli: NomadCli,
    s3: S3Client,
    config: PluginRegistryServiceConfig,
}

#[async_trait]
impl PluginRegistryApi for PluginRegistry {
    type Error = PluginRegistryServiceError;

    // TODO: This function is so long I'm gonna split it out into its own file soon.
    #[tracing::instrument(skip(self, request), err)]
    async fn create_plugin(
        &self,
        request: futures::channel::mpsc::Receiver<CreatePluginRequest>,
    ) -> Result<CreatePluginResponse, Self::Error> {
        let start_time = std::time::SystemTime::now();

        let mut request = request;

        let PluginMetadata {
            tenant_id,
            display_name,
            plugin_type,
            event_source_id,
        } = match request.next().await {
            Some(CreatePluginRequest::Metadata(m)) => m,
            _ => {
                return Err(Self::Error::StreamInputError(
                    "Expected request 0 to be Metadata",
                ));
            }
        };

        let plugin_id = generate_plugin_id();
        let s3_key = generate_artifact_s3_key(plugin_type, &tenant_id, &plugin_id);
        let s3_multipart_fields = create_plugin::S3MultipartFields {
            bucket: self.config.bucket_name.clone(),
            key: s3_key.clone(),
            expected_bucket_owner: Some(self.config.bucket_aws_account_id.clone()),
        };

        let multipart_upload =
            upload_stream_multipart_to_s3(request, &self.s3, &self.config, s3_multipart_fields)
                .await?;
        // Emit some benchmark info
        {
            let total_duration = std::time::SystemTime::now()
                .duration_since(start_time)
                .unwrap_or_default();

            tracing::info!(
                message = "CreatePlugin benchmark",
                display_name = ?display_name,
                duration_millis = ?total_duration.as_millis(),
                stream_length_bytes = multipart_upload.stream_length,
            );
        }

        self.db_client
            .create_plugin(
                &plugin_id,
                DbCreatePluginArgs {
                    tenant_id,
                    display_name,
                    plugin_type,
                    event_source_id,
                },
                &s3_key,
            )
            .await?;

        Ok(CreatePluginResponse { plugin_id })
    }

    #[tracing::instrument(skip(self, request), err)]
    async fn get_plugin(
        &self,
        request: GetPluginRequest,
    ) -> Result<GetPluginResponse, Self::Error> {
        let PluginRow {
            artifact_s3_key: _,
            plugin_type,
            plugin_id,
            display_name,
            tenant_id,
            event_source_id,
        } = self.db_client.get_plugin(&request.plugin_id).await?;

        let plugin_type: PluginType = try_from(&plugin_type)?;

        let response = GetPluginResponse {
            plugin_id,
            plugin_metadata: PluginMetadata {
                tenant_id,
                display_name,
                plugin_type,
                event_source_id,
            },
        };

        Ok(response)
    }

    #[tracing::instrument(skip(self, request), err)]
    async fn deploy_plugin(
        &self,
        request: DeployPluginRequest,
    ) -> Result<DeployPluginResponse, Self::Error> {
        let plugin_id = request.plugin_id;
        let plugin_row = self.db_client.get_plugin(&plugin_id).await?;

        // TODO: Given how many fields I'm forwarding here, it may just
        // make sense to pass `deploy_plugin` &self verbatim...
        deploy_plugin::deploy_plugin(
            &self.nomad_client,
            &self.nomad_cli,
            &self.db_client,
            plugin_row,
            &self.config,
        )
        .await
        .map_err(PluginRegistryServiceError::from)?;

        Ok(DeployPluginResponse {})
    }

    #[allow(dead_code)]
    #[tracing::instrument(skip(self, _request), err)]
    async fn tear_down_plugin(
        &self,
        _request: TearDownPluginRequest,
    ) -> Result<TearDownPluginResponse, Self::Error> {
        todo!()
    }

    #[tracing::instrument(skip(self, request), err)]
    async fn get_generators_for_event_source(
        &self,
        request: GetGeneratorsForEventSourceRequest,
    ) -> Result<GetGeneratorsForEventSourceResponse, Self::Error> {
        Ok(GetGeneratorsForEventSourceResponse {
            plugin_ids: self
                .db_client
                .get_generators_for_event_source(&request.event_source_id)
                .await?
                .iter()
                .map(|row| row.plugin_id)
                .collect(),
        })
    }

    #[allow(dead_code)]
    #[tracing::instrument(skip(self, _request), err)]
    async fn get_analyzers_for_tenant(
        &self,
        _request: GetAnalyzersForTenantRequest,
    ) -> Result<GetAnalyzersForTenantResponse, Self::Error> {
        todo!()
    }

    #[tracing::instrument(skip(self, request), err)]
    async fn get_plugin_health(
        &self,
        request: GetPluginHealthRequest,
    ) -> Result<GetPluginHealthResponse, Self::Error> {
        let health_status = get_plugin_health::get_plugin_health(
            &self.nomad_client,
            &self.db_client,
            request.plugin_id,
        );
        Ok(GetPluginHealthResponse { health_status })
    }
}

pub async fn exec_service(config: PluginRegistryConfig) -> Result<(), Box<dyn std::error::Error>> {
    let db_config = config.db_config;

    tracing::info!(
        message="Connecting to plugin registry table",
        plugin_registry_db_username=%db_config.plugin_registry_db_username,
        plugin_registry_db_hostname=%db_config.plugin_registry_db_hostname,
        plugin_registry_db_port=%db_config.plugin_registry_db_port,
    );
    let postgres_address = format!(
        "postgresql://{}:{}@{}:{}",
        db_config.plugin_registry_db_username,
        db_config.plugin_registry_db_password,
        db_config.plugin_registry_db_hostname,
        db_config.plugin_registry_db_port,
    );

    let addr = config.service_config.plugin_registry_bind_address;

    let plugin_registry = PluginRegistry {
        db_client: PluginRegistryDbClient::new(&postgres_address).await?,
        nomad_client: NomadClient::from_env(),
        nomad_cli: NomadCli::default(),
        s3: S3Client::from_env(),
        config: config.service_config,
    };

    let healthcheck_polling_interval_ms = 5000; // TODO: un-hardcode
    let (server, _shutdown_tx) = PluginRegistryServer::new(
        plugin_registry,
        TcpListener::bind(addr.clone()).await?,
        || async { Ok(HealthcheckStatus::Serving) }, // FIXME: this is garbage
        Duration::from_millis(healthcheck_polling_interval_ms),
    );
    tracing::info!(
        message = "starting gRPC server",
        socket_address = %addr,
    );

    Ok(server.serve().await?)
}

fn generate_artifact_s3_key(
    plugin_type: PluginType,
    tenant_id: &uuid::Uuid,
    plugin_id: &uuid::Uuid,
) -> String {
    format!(
        "plugins/tenant_id_{}/plugin_type-{}/{}.bin",
        tenant_id.as_hyphenated(),
        plugin_type.type_name(),
        plugin_id.as_hyphenated(),
    )
}

fn generate_plugin_id() -> uuid::Uuid {
    // NOTE: Previously we generated plugin-id based off of the plugin binary, but
    // since the binary is now streamed, + eventually 1 plugin can have different
    // versions - we decided to make it a random UUID.
    uuid::Uuid::new_v4()
}
