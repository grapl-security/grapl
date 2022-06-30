use std::{
    net::SocketAddr,
    time::Duration,
};

use futures::StreamExt;
use grapl_config::env_helpers::FromEnv;
use rusoto_s3::{
    GetObjectRequest,
    S3Client,
    S3,
};
use rust_proto::{
    graplinc::grapl::api::plugin_registry::v1beta1::{
        CreateAnalyzerRequest,
        CreateAnalyzerRequestMetadata,
        CreateGeneratorRequest,
        CreateGeneratorRequestMetadata,
        CreatePluginResponse,
        DeployPluginRequest,
        DeployPluginResponse,
        GetAnalyzersForTenantRequest,
        GetAnalyzersForTenantResponse,
        GetGeneratorsForEventSourceRequest,
        GetGeneratorsForEventSourceResponse,
        GetPluginRequest,
        GetPluginResponse,
        Plugin,
        PluginRegistryApi,
        PluginRegistryServer,
        PluginType,
        TearDownPluginRequest,
        TearDownPluginResponse,
    },
    protocol::healthcheck::HealthcheckStatus,
};
use tokio::{
    io::AsyncReadExt,
    net::TcpListener,
};
use tonic::async_trait;

use super::create_plugin::{
    upload_stream_multipart_to_s3,
    TryIntoMetadata,
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
    pub plugin_execution_container_image: String,
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

    #[tracing::instrument(skip(self, request), err)]
    async fn create_analyzer(
        &self,
        request: futures::channel::mpsc::Receiver<CreateAnalyzerRequest>,
    ) -> Result<CreatePluginResponse, Self::Error> {
        let mut request = request;

        let CreateAnalyzerRequestMetadata {
            tenant_id,
            display_name,
        } = request.next().await.try_into_metadata()?;

        let plugin_type = PluginType::Analyzer;
        let plugin_id = generate_plugin_id();
        let s3_key = generate_artifact_s3_key(plugin_type, &tenant_id, &plugin_id);
        let s3_multipart_fields = create_plugin::S3MultipartFields {
            bucket: self.config.bucket_name.clone(),
            key: s3_key.clone(),
            expected_bucket_owner: Some(self.config.bucket_aws_account_id.clone()),
        };

        upload_stream_multipart_to_s3(request, &self.s3, &self.config, s3_multipart_fields).await?;

        self.db_client
            .create_plugin(
                &plugin_id,
                DbCreatePluginArgs {
                    tenant_id,
                    display_name,
                    plugin_type,
                },
                &s3_key,
            )
            .await?;

        Ok(CreatePluginResponse { plugin_id })
    }

    #[tracing::instrument(skip(self, request), err)]
    async fn create_generator(
        &self,
        request: futures::channel::mpsc::Receiver<CreateGeneratorRequest>,
    ) -> Result<CreatePluginResponse, Self::Error> {
        let mut request = request;

        let CreateGeneratorRequestMetadata {
            tenant_id,
            display_name,
            event_sources,
        } = request.next().await.try_into_metadata()?;

        let plugin_type = PluginType::Generator;
        let plugin_id = generate_plugin_id();
        let s3_key = generate_artifact_s3_key(plugin_type, &tenant_id, &plugin_id);
        let s3_multipart_fields = create_plugin::S3MultipartFields {
            bucket: self.config.bucket_name.clone(),
            key: s3_key.clone(),
            expected_bucket_owner: Some(self.config.bucket_aws_account_id.clone()),
        };

        upload_stream_multipart_to_s3(request, &self.s3, &self.config, s3_multipart_fields).await?;

        self.db_client
            .create_plugin(
                &plugin_id,
                DbCreatePluginArgs {
                    tenant_id,
                    display_name,
                    plugin_type,
                },
                &s3_key,
            )
            .await?;

        // TODO: A DB client call that adds a one-to-many for event sources
        // that are associated with this Generator
        let _event_sources = event_sources;

        Ok(CreatePluginResponse { plugin_id })
    }

    #[tracing::instrument(skip(self, request), err)]
    async fn get_plugin(
        &self,
        request: GetPluginRequest,
    ) -> Result<GetPluginResponse, Self::Error> {
        let PluginRow {
            artifact_s3_key,
            plugin_type,
            plugin_id,
            display_name,
            tenant_id: _,
        } = self.db_client.get_plugin(&request.plugin_id).await?;

        let s3_key: String = artifact_s3_key;
        let plugin_type: PluginType = try_from(&plugin_type)?;

        let get_object_output = self
            .s3
            .get_object(GetObjectRequest {
                bucket: self.config.bucket_name.clone(),
                key: s3_key.clone(),
                expected_bucket_owner: Some(self.config.bucket_aws_account_id.clone()),
                ..Default::default()
            })
            .await?;

        let stream = get_object_output
            .body
            .ok_or(PluginRegistryServiceError::EmptyObject)?;

        let mut plugin_binary = Vec::new();

        // read the whole file
        stream
            .into_async_read()
            .read_to_end(&mut plugin_binary)
            .await?;

        let response = GetPluginResponse {
            plugin: Plugin {
                plugin_id,
                display_name,
                plugin_type,
                plugin_binary,
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

    #[tracing::instrument(skip(self, _request), err)]
    async fn get_generators_for_event_source(
        &self,
        _request: GetGeneratorsForEventSourceRequest,
    ) -> Result<GetGeneratorsForEventSourceResponse, Self::Error> {
        todo!()
    }

    #[allow(dead_code)]
    #[tracing::instrument(skip(self, _request), err)]
    async fn get_analyzers_for_tenant(
        &self,
        _request: GetAnalyzersForTenantRequest,
    ) -> Result<GetAnalyzersForTenantResponse, Self::Error> {
        todo!()
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

    server.serve().await
}

fn generate_artifact_s3_key(
    plugin_type: PluginType,
    tenant_id: &uuid::Uuid,
    plugin_id: &uuid::Uuid,
) -> String {
    format!(
        "plugins/tenant_id_{}/plugin_type-{}/{}.bin",
        tenant_id.to_hyphenated(),
        plugin_type.type_name(),
        plugin_id.to_hyphenated(),
    )
}

fn generate_plugin_id() -> uuid::Uuid {
    // NOTE: Previously we generated plugin-id based off of the plugin binary, but
    // since the binary is now streamed, + eventually 1 plugin can have different
    // versions - we decided to make it a random UUID.
    uuid::Uuid::new_v4()
}
