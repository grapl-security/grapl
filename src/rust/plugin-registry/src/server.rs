use grapl_config::env_helpers::FromEnv;
use grapl_utils::future_ext::GraplFutureExt;
use rusoto_s3::{
    GetObjectError,
    GetObjectRequest,
    PutObjectError,
    PutObjectRequest,
    S3Client,
    S3,
};
use rust_proto::plugin_registry::{
    plugin_registry_service_server::{
        PluginRegistryService,
        PluginRegistryServiceServer,
    },
    CreatePluginRequest,
    CreatePluginRequestProto,
    CreatePluginResponse,
    CreatePluginResponseProto,
    DeployPluginRequest,
    DeployPluginRequestProto,
    DeployPluginResponse,
    DeployPluginResponseProto,
    GetAnalyzersForTenantRequest,
    GetAnalyzersForTenantRequestProto,
    GetAnalyzersForTenantResponse,
    GetAnalyzersForTenantResponseProto,
    GetGeneratorsForEventSourceRequest,
    GetGeneratorsForEventSourceRequestProto,
    GetGeneratorsForEventSourceResponse,
    GetGeneratorsForEventSourceResponseProto,
    GetPluginRequest,
    GetPluginRequestProto,
    GetPluginResponse,
    GetPluginResponseProto,
    Plugin,
    PluginRegistryDeserializationError,
    PluginType,
    TearDownPluginRequest,
    TearDownPluginRequestProto,
    TearDownPluginResponse,
    TearDownPluginResponseProto,
};
use tonic::{
    transport::Server,
    Request,
    Response,
    Status,
};

#[derive(sqlx::FromRow)]
struct GetPluginRow {
    plugin_id: uuid::Uuid,
    display_name: String,
    plugin_type: String,
    artifact_s3_key: String,
}

use tokio::io::AsyncReadExt;

use crate::PluginRegistryServiceConfig;

#[derive(Debug, thiserror::Error)]
pub enum PluginRegistryServiceError {
    #[error("SqlxError")]
    SqlxError(#[from] sqlx::Error),
    #[error("S3PutObjectError")]
    PutObjectError(#[from] rusoto_core::RusotoError<PutObjectError>),
    #[error("S3GetObjectError")]
    GetObjectError(#[from] rusoto_core::RusotoError<GetObjectError>),
    #[error("EmptyObject")]
    EmptyObject,
    #[error("IoError")]
    IoError(#[from] std::io::Error),
    #[error("PluginRegistryDeserializationError")]
    PluginRegistryDeserializationError(#[from] PluginRegistryDeserializationError),
}

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
            PluginRegistryServiceError::PluginRegistryDeserializationError(_) => {
                Status::invalid_argument("Unable to deserialize message")
            }
        }
    }
}

pub struct PluginRegistry {
    pool: sqlx::PgPool,
    s3: S3Client,
    plugin_bucket_name: String,
    plugin_bucket_owner_id: String,
}

impl PluginRegistry {
    async fn create_plugin(
        &self,
        request: CreatePluginRequest,
    ) -> Result<CreatePluginResponse, PluginRegistryServiceError> {
        let plugin_id = generate_plugin_id(&request.tenant_id, request.plugin_artifact.as_slice());

        let s3_key = generateo_artifact_s3_key(request.plugin_type, &request.tenant_id, &plugin_id);

        self.s3
            .put_object(PutObjectRequest {
                content_length: Some(request.plugin_artifact.len() as i64),
                body: Some(request.plugin_artifact.into()),
                bucket: self.plugin_bucket_name.clone(),
                key: s3_key.clone(),
                expected_bucket_owner: Some(self.plugin_bucket_owner_id.clone()),
                ..Default::default()
            })
            .await?;

        sqlx::query(
            r"
            INSERT INTO plugins (
                plugin_id,
                plugin_type,
                display_name,
                tenant_id,
                artifact_s3_key
            )
            VALUES ($1::uuid, $2, $3, $4::uuid, $5)
            ON CONFLICT DO NOTHING;
            ",
        )
        .bind(plugin_id)
        .bind(request.plugin_type.type_name())
        .bind(request.display_name)
        .bind(request.tenant_id)
        .bind(s3_key)
        .execute(&self.pool)
        .await?;

        let response = CreatePluginResponse { plugin_id };
        Ok(response)
    }

    #[tracing::instrument(skip(self, request), err)]
    async fn get_plugin(
        &self,
        request: GetPluginRequest,
    ) -> Result<GetPluginResponse, PluginRegistryServiceError> {
        let row: GetPluginRow = sqlx::query_as(
            r"
        SELECT
        plugin_id,
        display_name,
        plugin_type,
        artifact_s3_key
        FROM plugins
        WHERE plugin_id = $1
            ",
        )
        .bind(request.plugin_id)
        .fetch_one(&self.pool)
        .await?;

        let GetPluginRow {
            plugin_id,
            display_name,
            plugin_type,
            artifact_s3_key,
        } = row;
        let s3_key: String = artifact_s3_key;
        let plugin_type: PluginType = PluginType::try_from(plugin_type)?;

        let get_object_output = self
            .s3
            .get_object(GetObjectRequest {
                bucket: self.plugin_bucket_name.clone(),
                key: s3_key.clone(),
                expected_bucket_owner: Some(self.plugin_bucket_owner_id.clone()),
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

    #[allow(dead_code)]
    async fn deploy_plugin(
        &self,
        _request: DeployPluginRequest,
    ) -> Result<DeployPluginResponse, PluginRegistryServiceError> {
        todo!()
    }

    #[allow(dead_code)]
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
    async fn get_analyzers_for_tenant(
        &self,
        _request: GetAnalyzersForTenantRequest,
    ) -> Result<GetAnalyzersForTenantResponse, PluginRegistryServiceError> {
        todo!()
    }
}

#[async_trait::async_trait]
impl PluginRegistryService for PluginRegistry {
    async fn create_plugin(
        &self,
        request: Request<CreatePluginRequestProto>,
    ) -> Result<Response<CreatePluginResponseProto>, Status> {
        let request: CreatePluginRequestProto = request.into_inner();
        let request: CreatePluginRequest =
            CreatePluginRequest::try_from(request).map_err(PluginRegistryServiceError::from)?;

        let response = self.create_plugin(request).await?;
        let response: CreatePluginResponseProto = response.into();
        Ok(Response::new(response))
    }

    async fn get_plugin(
        &self,
        request: Request<GetPluginRequestProto>,
    ) -> Result<Response<GetPluginResponseProto>, Status> {
        let request: GetPluginRequestProto = request.into_inner();
        let request =
            GetPluginRequest::try_from(request).map_err(PluginRegistryServiceError::from)?;

        let response = self.get_plugin(request).await?;
        let response: GetPluginResponseProto = response.into();
        Ok(Response::new(response))
    }

    async fn deploy_plugin(
        &self,
        _request: Request<DeployPluginRequestProto>,
    ) -> Result<Response<DeployPluginResponseProto>, Status> {
        todo!()
    }

    async fn tear_down_plugin(
        &self,
        _request: Request<TearDownPluginRequestProto>,
    ) -> Result<Response<TearDownPluginResponseProto>, Status> {
        todo!()
    }

    #[tracing::instrument(skip(self, request), err)]
    async fn get_generators_for_event_source(
        &self,
        request: Request<GetGeneratorsForEventSourceRequestProto>,
    ) -> Result<Response<GetGeneratorsForEventSourceResponseProto>, Status> {
        let request = request.into_inner();
        let _request = GetGeneratorsForEventSourceRequest::try_from(request)
            .map_err(PluginRegistryServiceError::from)?;
        todo!()
    }

    async fn get_analyzers_for_tenant(
        &self,
        _request: Request<GetAnalyzersForTenantRequestProto>,
    ) -> Result<Response<GetAnalyzersForTenantResponseProto>, Status> {
        todo!()
    }
}

pub async fn exec_service(
    service_config: PluginRegistryServiceConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter
        .set_serving::<PluginRegistryServiceServer<PluginRegistry>>()
        .await;

    tracing::info!(
        message="Connecting to plugin registry table",
        plugin_registry_db_username=%service_config.plugin_registry_db_username,
        plugin_registry_db_hostname=%service_config.plugin_registry_db_hostname,
        plugin_registry_db_port=%service_config.plugin_registry_db_port,
    );
    let postgres_address = format!(
        "postgresql://{}:{}@{}:{}",
        service_config.plugin_registry_db_username,
        service_config.plugin_registry_db_password,
        service_config.plugin_registry_db_hostname,
        service_config.plugin_registry_db_port,
    );

    let plugin_registry: PluginRegistry = PluginRegistry {
        pool: sqlx::PgPool::connect(&postgres_address)
            .timeout(std::time::Duration::from_secs(5))
            .await??,
        s3: S3Client::from_env(),
        plugin_bucket_name: service_config.plugin_s3_bucket_name,
        plugin_bucket_owner_id: service_config.plugin_s3_bucket_aws_account_id,
    };

    sqlx::migrate!().run(&plugin_registry.pool).await?;

    let addr = service_config.plugin_registry_bind_address;
    tracing::info!(
        message="Starting PluginRegistry",
        addr=?addr,
    );

    Server::builder()
        .trace_fn(|request| {
            tracing::info_span!(
                "PluginRegistry",
                headers = ?request.headers(),
                method = ?request.method(),
                uri = %request.uri(),
                extensions = ?request.extensions(),
            )
        })
        .add_service(health_service)
        .add_service(PluginRegistryServiceServer::new(plugin_registry))
        .serve(addr)
        .await?;

    Ok(())
}

fn generateo_artifact_s3_key(
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
