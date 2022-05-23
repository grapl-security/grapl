use std::{
    net::SocketAddr,
    sync::{
        atomic::{
            AtomicUsize,
            Ordering,
        },
        Arc,
    },
    time::Duration,
};

use bytes::Bytes;
use futures::StreamExt;
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
        CreatePluginRequest,
        CreatePluginRequestMetadata,
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

fn io_input_error(input: impl std::fmt::Display) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidInput, input.to_string())
}

/// Keep adding the size of an incoming chunk to an AtomicUsize.
/// Start throwing errors when it exceeds limit.
async fn accumulate_stream_size_to_limit(
    stream_length: Arc<AtomicUsize>,
    limit: usize,
    result: Result<Bytes, std::io::Error>,
) -> Result<Bytes, std::io::Error> {
    match result {
        Ok(bytes) => {
            if stream_length.fetch_add(bytes.len(), Ordering::SeqCst) > limit {
                Err(io_input_error(format!("Input exceeds limit {limit} bytes")))
            } else {
                Ok(bytes)
            }
        }
        Err(e) => Err(e),
    }
}

#[async_trait]
impl PluginRegistryApi for PluginRegistry {
    type Error = PluginRegistryServiceError;

    #[tracing::instrument(skip(self, request), err)]
    async fn create_plugin(
        &self,
        request: futures::channel::mpsc::Receiver<CreatePluginRequest>,
    ) -> Result<CreatePluginResponse, Self::Error> {
        let start_time = std::time::SystemTime::now();

        let mut request = request;

        let CreatePluginRequestMetadata {
            tenant_id,
            display_name,
            plugin_type,
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

        // Convert the incoming request stream of CreatePluginRequest::Chunk
        // into a stream of Bytes to be sent to Rusoto S3.put_object.
        let body_stream = request.then(|result| async {
            match result {
                CreatePluginRequest::Chunk(c) => Ok(Bytes::from(c)),
                _ => Err(io_input_error("Expected request 1..N to be Chunk")),
            }
        });

        // While Rusoto reads the stream, keep track of total legth of chunks.
        // Bail out if we exceed the limit specified in the config.
        let limit_bytes = self.config.artifact_size_limit_mb.clone() * 1024 * 1024;
        let stream_length = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        // Get a handle for the Stream to move
        let stream_length_handle = stream_length.clone();
        let body_stream = body_stream.then(move |result| {
            accumulate_stream_size_to_limit(stream_length_handle.clone(), limit_bytes, result)
        });

        // Send the Stream into Rusoto, which does the actual awaiting of each
        // future in the Stream.
        self.s3
            .put_object(PutObjectRequest {
                body: Some(StreamingBody::new(body_stream)),
                bucket: self.config.bucket_name.clone(),
                key: s3_key.clone(),
                expected_bucket_owner: Some(self.config.bucket_aws_account_id.clone()),
                ..Default::default()
            })
            .await?;

        // Emit some benchmark info
        {
            let stream_length: usize = Arc::try_unwrap(stream_length)
                .expect("Stream has been exhausted by this point")
                .into_inner();
            let total_duration = std::time::SystemTime::now()
                .duration_since(start_time)
                .unwrap_or_default();

            tracing::info!(
                message = "CreatePlugin benchmark",
                display_name = ?display_name,
                duration_millis = ?total_duration.as_millis(),
                size_mb = stream_length / (1024 * 1024)
            );
        }

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
