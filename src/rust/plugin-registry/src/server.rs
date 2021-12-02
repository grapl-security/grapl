#![allow(warnings)]

use rusoto_s3::{
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

#[derive(Debug, thiserror::Error)]
pub enum PluginRegistryServiceError {}

pub struct PluginRegistry {
    pool: sqlx::PgPool,
    s3: S3Client,
}

impl PluginRegistry {
    #[allow(dead_code)]
    async fn create_plugin(
        &self,
        _request: CreatePluginRequest,
    ) -> Result<CreatePluginResponse, PluginRegistryServiceError> {
        let mut hasher = blake3::Hasher::new();
        hasher.update(_request.tenant_id.as_bytes());
        hasher.update(_request.plugin_artifact.as_slice());
        let artifact_id = hasher.finalize().to_hex();

        let s3_key = format!(
            "bucketname/{}/{}-plugins/{}.bin",
            _request.tenant_id,
            _request.plugin_type.type_name(),
            &artifact_id,
        );

        self.s3.put_object(PutObjectRequest {
            content_length: Some(_request.plugin_artifact.len() as i64),
            body: Some(_request.plugin_artifact.into()),
            bucket: "todo!".to_string(),
            key: s3_key.clone(),
            expected_bucket_owner: Some("todo!".to_string()),
            metadata: None,
            ..Default::default()
        });

        sqlx::query(
            r"
            INSERT INTO plugin_artifacts (
                artifact_id,
                artifact_version,
                artifact_s3_key
            )
            VALUES ($1, $2, $3)
            ON CONFLICT DO NOTHING;
            ",
        )
        .bind(artifact_id.as_str())
        .bind(0)
        .bind(s3_key)
        .fetch_one(&self.pool)
        .await
        .expect("todo");

        todo!()
    }

    #[allow(dead_code)]
    async fn get_plugin(
        &self,
        _request: GetPluginRequest,
    ) -> Result<GetPluginResponse, PluginRegistryServiceError> {
        todo!()
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

    #[allow(dead_code)]
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
        _request: Request<CreatePluginRequestProto>,
    ) -> Result<Response<CreatePluginResponseProto>, Status> {
        todo!()
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

    async fn get_generators_for_event_source(
        &self,
        _request: Request<GetGeneratorsForEventSourceRequestProto>,
    ) -> Result<Response<GetGeneratorsForEventSourceResponseProto>, Status> {
        todo!()
    }

    async fn get_analyzers_for_tenant(
        &self,
        _request: Request<GetAnalyzersForTenantRequestProto>,
    ) -> Result<Response<GetAnalyzersForTenantResponseProto>, Status> {
        todo!()
    }
}

pub async fn exec_service() -> Result<(), Box<dyn std::error::Error>> {
    let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter
        .set_serving::<PluginRegistryServiceServer<PluginRegistry>>()
        .await;

    let addr = "[::1]:50051".parse().unwrap();
    let plugin_work_queue: PluginRegistry = todo!();

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
        .add_service(PluginRegistryServiceServer::new(plugin_work_queue))
        .serve(addr)
        .await?;

    Ok(())
}
