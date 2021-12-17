use grapl_config::env_helpers::FromEnv;
use grapl_utils::future_ext::GraplFutureExt;
use rusoto_s3::{PutObjectRequest, S3Client, S3, GetObjectRequest};
use sqlx::Row;
use rust_proto::plugin_registry::{plugin_registry_service_server::{
    PluginRegistryService,
    PluginRegistryServiceServer,
}, CreatePluginRequest, CreatePluginRequestProto, CreatePluginResponse, CreatePluginResponseProto, DeployPluginRequest, DeployPluginRequestProto, DeployPluginResponse, DeployPluginResponseProto, GetAnalyzersForTenantRequest, GetAnalyzersForTenantRequestProto, GetAnalyzersForTenantResponse, GetAnalyzersForTenantResponseProto, GetGeneratorsForEventSourceRequest, GetGeneratorsForEventSourceRequestProto, GetGeneratorsForEventSourceResponse, GetGeneratorsForEventSourceResponseProto, GetPluginRequest, GetPluginRequestProto, GetPluginResponse, GetPluginResponseProto, TearDownPluginRequest, TearDownPluginRequestProto, TearDownPluginResponse, TearDownPluginResponseProto, PluginType, Plugin};
use tonic::{
    transport::Server,
    Request,
    Response,
    Status,
};

use tokio::io::{AsyncReadExt};

use crate::PluginRegistryServiceConfig;

#[derive(Debug, thiserror::Error)]
pub enum PluginRegistryServiceError {}

impl From<PluginRegistryServiceError> for Status {
    fn from(err: PluginRegistryServiceError) -> Self {
        match err {}
    }
}

pub struct PluginRegistry {
    pool: sqlx::PgPool,
    s3: S3Client,
    plugin_bucket_name: String,
    plugin_bucket_owner_id: String,
}

impl PluginRegistry {
    #[allow(dead_code)]
    async fn create_plugin(
        &self,
        request: CreatePluginRequest,
    ) -> Result<CreatePluginResponse, PluginRegistryServiceError> {
        let mut hasher = blake3::Hasher::new();
        hasher.update(request.tenant_id.as_bytes());
        hasher.update(request.plugin_artifact.as_slice());
        let artifact_id = hasher.finalize().to_hex();

        let s3_key = format!(
            "bucketname/{}/{}-plugins/{}.bin",
            request.tenant_id,
            request.plugin_type.type_name(),
            &artifact_id,
        );

        self.s3
            .put_object(PutObjectRequest {
                content_length: Some(request.plugin_artifact.len() as i64),
                body: Some(request.plugin_artifact.into()),
                bucket: self.plugin_bucket_name.clone(),
                key: s3_key.clone(),
                expected_bucket_owner: Some(self.plugin_bucket_owner_id.clone()),
                metadata: None,
                ..Default::default()
            })
            .await
            .expect("Failed to put_object");

        let response = sqlx::query(
            r"
            INSERT INTO plugin_artifacts (
                artifact_id,
                artifact_version,
                artifact_s3_key,
                plugin_type,
                tenant_id,
            )
            VALUES ($1, $2, $3, $4, $5)
            SELECT (plugin_id)
            ON CONFLICT DO NOTHING;
            ",
        )
        .bind(artifact_id.as_str())
        .bind(0) // todo: Artifact versioning
        .bind(s3_key)
        .bind(request.plugin_type.type_name())
        .bind(request.tenant_id)
        .fetch_one(&self.pool)
        .await
        .expect("todo");

        let plugin_id: uuid::Uuid = response.try_get("plugin_id").expect("todo");

        let response = CreatePluginResponse {
            plugin_id,
        };
        Ok(response)
    }

    #[allow(dead_code)]
    async fn get_plugin(
        &self,
        request: GetPluginRequest,
    ) -> Result<GetPluginResponse, PluginRegistryServiceError> {

        let row = sqlx::query(
            r"
            SELECT plugin_id, display_name, artifact_id, plugin_type FROM plugin_artifacts
            WHERE plugin_id = ?;
            ",
        )
            .bind(request.plugin_id)
            .fetch_one(&self.pool)
            .await
            .expect("todo");

        let plugin_id: uuid::Uuid = row.try_get("plugin_id").expect("todo");
        // todo: Validate that the request plugin_id matches the response plugin_id
        let display_name: String = row.try_get("display_name").expect("todo");
        let artifact_id: String = row.try_get("artifact_id").expect("todo");
        let plugin_type: String = row.try_get("plugin_type").expect("todo");
        let plugin_type: PluginType = PluginType::try_from(plugin_type).expect("todo");

        let s3_key = format!(
            "bucketname/{}/{}-plugins/{}.bin",
            request.tenant_id,
            plugin_type.type_name(),
            &artifact_id,
        );

        let get_object_output = self.s3
            .get_object(GetObjectRequest {
                bucket: self.plugin_bucket_name.clone(),
                key: s3_key.clone(),
                expected_bucket_owner: Some(self.plugin_bucket_owner_id.clone()),
                ..Default::default()
            })
            .await
            .expect("Failed to put_object");

        let stream = get_object_output.body.expect("todo");

        let mut buffer = Vec::new();

        // read the whole file
        stream.into_async_read().read_to_end(&mut buffer).await
            .expect("todo");

        let response = GetPluginResponse {
            plugin: Plugin {
                plugin_id,
                display_name,
                plugin_type
            }
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
        let request: CreatePluginRequest = CreatePluginRequest::try_from(request)
            .expect("todo");

        let response = self.create_plugin(request).await.expect("todo");
        let response: CreatePluginResponseProto = response.into();
        Ok(Response::new(response))
    }

    async fn get_plugin(
        &self,
        _request: Request<GetPluginRequestProto>,
    ) -> Result<Response<GetPluginResponseProto>, Status> {
        todo!()
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
        let request =
            GetGeneratorsForEventSourceRequest::try_from(request).expect("Invalid message");

        match self.get_generators_for_event_source(request).await {
            Ok(response) => {
                tracing::debug!(
                    message="Successfully retrieved generators for event source",
                    plugin_ids=?response.plugin_ids,
                );
                Ok(Response::new(
                    GetGeneratorsForEventSourceResponseProto::from(response),
                ))
            }
            Err(e) => {
                tracing::warn!(
                    message="Failed to get get_generators_for_event_source",
                    error=?e,
                );
                Err(Status::from(e))
            }
        }
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

    let addr = service_config.plugin_registry_bind_address;
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

    sqlx::migrate!()
        .run(&plugin_registry.pool)
        .await?;

    tracing::info!(
        message="HealthServer + PluginRegistry listening",
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
