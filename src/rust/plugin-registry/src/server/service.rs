use std::{
    net::SocketAddr,
    time::Duration, pin::Pin,
};

use bytes::Bytes;
use futures::{Stream, StreamExt, stream, channel::mpsc, SinkExt};
use grapl_config::env_helpers::FromEnv;
use rusoto_s3::{
    GetObjectRequest,
    PutObjectRequest,
    S3Client,
    StreamingBody,
    S3,
};
use rust_proto_new::{
    graplinc::grapl::api::plugin_registry::v1beta1::{
        CreatePluginRequestV2,
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
        TearDownPluginResponse, CreatePluginRequestMetadata,
    },
    protocol::{healthcheck::HealthcheckStatus, status::Status},
};
use tokio::{
    io::AsyncReadExt,
    net::TcpListener,
};
use tonic::async_trait;

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
    server::deploy_plugin,
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

impl PluginRegistry {
    fn ensure_artifact_size_limit(&self, bytes: &[u8]) -> Result<(), PluginRegistryServiceError> {
        let limit_mb = &self.config.artifact_size_limit_mb;
        if bytes.len() > (limit_mb * 1024 * 1024) {
            Err(PluginRegistryServiceError::ArtifactTooLargeError(format!(
                "Artifact exceeds {limit_mb}MB"
            )))
        } else {
            Ok(())
        }
    }
}
type PinnedStream<T> = Pin<Box<dyn Stream<Item = T> + Send + 'static>>;
type ResultStream<T, E> = PinnedStream<Result<T, E>>;

// Roughly equivalent to `return Err(...)` in a fn -> Result<T, E>
fn err_stream <T, E> (error: E) -> ResultStream<T, Status> 
where 
    T: Sync + Send + 'static,
    E: Sync + Send + 'static + Into<Status>
{
    Box::pin(
        stream::iter(vec![Err(error.into())])
    )
}

#[async_trait]
impl PluginRegistryApi for PluginRegistry {
    type Error = PluginRegistryServiceError;

    #[tracing::instrument(skip(self, request), err)]
    async fn create_plugin(
        &self,
        request: ResultStream<CreatePluginRequestV2, Status>
    ) -> Result<CreatePluginResponse, Self::Error>
    {
        type Req = CreatePluginRequestV2;

        let CreatePluginRequestMetadata {
            tenant_id,
            display_name,
            plugin_type,
        } = match request.next().await {
            Some(Ok(Req::Metadata(m))) => m,
            Some(Err(e)) => {
                return Err(e);
            }
            _ => {
                return Err(Self::Error::StreamInputError(
                    "Expected request 0 to be Metadata"
                ));
            }
        };

        let hash: Vec<u8> = "asdf".into();
        let plugin_id = generate_plugin_id(&tenant_id, hash.as_slice());

        let s3_key = generate_artifact_s3_key(plugin_type, &tenant_id, &plugin_id);

        let body_stream: ResultStream<Bytes, std::io::Error> = Box::pin(request.then(|result| async move {
            match result {
                Ok(Req::Chunk(c)) => Ok(Bytes::from(c)),
                _ => {
                    Err(std::io::Error::new(std::io::ErrorKind::Other,
                        "Expected request 1..N to be Chunk".to_string()
                    ))
                }
            }
        }));

        let s3_put_result = self.s3
            .put_object(PutObjectRequest {
                //content_length: Some(plugin_artifact.len() as i64),
                body: Some(StreamingBody::new(body_stream)),
                bucket: self.config.plugin_s3_bucket_name.clone(),
                key: s3_key.clone(),
                expected_bucket_owner: Some(self.config.plugin_s3_bucket_aws_account_id.clone()),
                ..Default::default()
            })
            .await?;

        let db_create_result = self.db_client
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
        Ok(CreatePluginResponse {
            plugin_id
        })
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

fn generate_plugin_id(tenant_id: &uuid::Uuid, plugin_artifact: &[u8]) -> uuid::Uuid {
    let mut hasher = blake3::Hasher::new();
    hasher.update(&b"PLUGIN_ID_NAMESPACE"[..]);
    hasher.update(tenant_id.as_bytes());
    hasher.update(plugin_artifact);
    let mut output = [0; 16];
    hasher.finalize_xof().fill(&mut output);

    uuid::Uuid::from_bytes(output)
}
