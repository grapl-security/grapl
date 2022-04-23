use std::{
    net::SocketAddr,
    time::Duration,
};

use grapl_config::env_helpers::FromEnv;
use rusoto_s3::{
    GetObjectRequest,
    PutObjectRequest,
    S3Client,
    S3,
};
use rust_proto_new::graplinc::grapl::api::plugin_registry::v1beta1::{
    CreatePluginRequest,
    CreatePluginResponse,
    DeployPluginRequest,
    DeployPluginResponse,
    GetAnalyzersForTenantRequest,
    GetAnalyzersForTenantResponse,
    GetGeneratorsForEventSourceRequest,
    GetGeneratorsForEventSourceResponse,
    GetPluginRequest,
    GetPluginResponse,
    HealthcheckStatus,
    Plugin,
    PluginRegistryApi,
    PluginRegistryServer,
    PluginType,
    TearDownPluginRequest,
    TearDownPluginResponse,
};
use structopt::StructOpt;
use tokio::{
    io::AsyncReadExt,
    net::TcpListener,
};
use tonic::{
    async_trait,
    Status,
};

use crate::{
    db::{
        client::PluginRegistryDbClient,
        models::PluginRow,
    },
    error::PluginRegistryServiceError,
    nomad::{
        cli::NomadCli,
        client::NomadClient,
    },
    server::deploy_plugin,
};

impl From<PluginRegistryServiceError> for Status {
    fn from(err: PluginRegistryServiceError) -> Self {
        match err {
            PluginRegistryServiceError::SqlxError(sqlx::Error::Configuration(_)) => {
                Status::internal("Invalid SQL configuration")
            }
            PluginRegistryServiceError::SqlxError(_) => {
                Status::internal("Failed to operate on postgres")
            }
            PluginRegistryServiceError::PutObjectError(_) => {
                Status::internal("Failed to put s3 object")
            }
            PluginRegistryServiceError::GetObjectError(_) => {
                Status::internal("Failed to get s3 object")
            }
            PluginRegistryServiceError::EmptyObject => {
                Status::internal("S3 Object was unexpectedly empty")
            }
            PluginRegistryServiceError::IoError(_) => Status::internal("IoError"),
            PluginRegistryServiceError::SerDeError(_) => {
                Status::invalid_argument("Unable to deserialize message")
            }
            PluginRegistryServiceError::NomadClientError(_) => {
                Status::internal("Failed RPC with Nomad")
            }
            PluginRegistryServiceError::NomadCliError(_) => {
                Status::internal("Failed using Nomad CLI")
            }
            PluginRegistryServiceError::NomadJobAllocationError => {
                Status::internal("Unable to allocate Nomad job - it may be out of resources.")
            }
        }
    }
}
#[derive(StructOpt, Debug)]
pub struct PluginRegistryConfig {
    #[structopt(flatten)]
    db_config: PluginRegistryDbConfig,
    #[structopt(flatten)]
    service_config: PluginRegistryServiceConfig,
}

#[derive(StructOpt, Debug)]
pub struct PluginRegistryDbConfig {
    #[structopt(env)]
    plugin_registry_db_hostname: String,
    #[structopt(env)]
    plugin_registry_db_port: u16,
    #[structopt(env)]
    plugin_registry_db_username: String,
    #[structopt(env)]
    plugin_registry_db_password: String,
}

#[derive(StructOpt, Debug)]
pub struct PluginRegistryServiceConfig {
    #[structopt(env)]
    pub plugin_s3_bucket_aws_account_id: String,
    #[structopt(env)]
    pub plugin_s3_bucket_name: String,
    #[structopt(env)]
    pub plugin_registry_bind_address: SocketAddr,
    #[structopt(env)]
    pub plugin_bootstrap_container_image: String,
    #[structopt(env)]
    pub plugin_execution_container_image: String,
    #[structopt(env = "PLUGIN_REGISTRY_KERNEL_ARTIFACT_URL")]
    pub kernel_artifact_url: String,
    #[structopt(env = "PLUGIN_REGISTRY_ROOTFS_ARTIFACT_URL")]
    pub rootfs_artifact_url: String,
    #[structopt(env = "PLUGIN_REGISTRY_HAX_DOCKER_PLUGIN_RUNTIME_IMAGE")]
    pub hax_docker_plugin_runtime_image: String,
}

pub struct PluginRegistry {
    db_client: PluginRegistryDbClient,
    nomad_client: NomadClient,
    nomad_cli: NomadCli,
    s3: S3Client,
    config: PluginRegistryServiceConfig,
}

#[async_trait]
impl PluginRegistryApi<PluginRegistryServiceError> for PluginRegistry {
    #[tracing::instrument(skip(self, request), err)]
    async fn create_plugin(
        &self,
        request: CreatePluginRequest,
    ) -> Result<CreatePluginResponse, PluginRegistryServiceError> {
        let plugin_id = generate_plugin_id(&request.tenant_id, request.plugin_artifact.as_slice());

        let s3_key = generate_artifact_s3_key(request.plugin_type, &request.tenant_id, &plugin_id);

        self.s3
            .put_object(PutObjectRequest {
                content_length: Some(request.plugin_artifact.len() as i64),
                body: Some(request.plugin_artifact.clone().into()),
                bucket: self.config.plugin_s3_bucket_name.clone(),
                key: s3_key.clone(),
                expected_bucket_owner: Some(self.config.plugin_s3_bucket_aws_account_id.clone()),
                ..Default::default()
            })
            .await?;

        self.db_client
            .create_plugin(&plugin_id, &request, &s3_key)
            .await?;

        let response = CreatePluginResponse { plugin_id };
        Ok(response)
    }

    #[tracing::instrument(skip(self, request), err)]
    async fn get_plugin(
        &self,
        request: GetPluginRequest,
    ) -> Result<GetPluginResponse, PluginRegistryServiceError> {
        let PluginRow {
            artifact_s3_key,
            plugin_type,
            plugin_id,
            display_name,
            tenant_id: _,
        } = self.db_client.get_plugin(&request.plugin_id).await?;

        let s3_key: String = artifact_s3_key;
        let plugin_type: PluginType = PluginType::try_from(plugin_type)?;

        let get_object_output = self
            .s3
            .get_object(GetObjectRequest {
                bucket: self.config.plugin_s3_bucket_name.clone(),
                key: s3_key.clone(),
                expected_bucket_owner: Some(self.config.plugin_s3_bucket_aws_account_id.clone()),
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
    ) -> Result<DeployPluginResponse, PluginRegistryServiceError> {
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
    ) -> Result<TearDownPluginResponse, PluginRegistryServiceError> {
        todo!()
    }

    #[tracing::instrument(skip(self, _request), err)]
    async fn get_generators_for_event_source(
        &self,
        _request: GetGeneratorsForEventSourceRequest,
    ) -> Result<GetGeneratorsForEventSourceResponse, PluginRegistryServiceError> {
        todo!()
    }

    #[allow(dead_code)]
    #[tracing::instrument(skip(self, _request), err)]
    async fn get_analyzers_for_tenant(
        &self,
        _request: GetAnalyzersForTenantRequest,
    ) -> Result<GetAnalyzersForTenantResponse, PluginRegistryServiceError> {
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

fn generate_plugin_id(tenant_id: &uuid::Uuid, plugin_artifact: &[u8]) -> uuid::Uuid {
    let mut hasher = blake3::Hasher::new();
    hasher.update(&b"PLUGIN_ID_NAMESPACE"[..]);
    hasher.update(tenant_id.as_bytes());
    hasher.update(plugin_artifact);
    let mut output = [0; 16];
    hasher.finalize_xof().fill(&mut output);

    uuid::Uuid::from_bytes(output)
}
