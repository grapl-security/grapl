use grapl_config::env_helpers::FromEnv;
use rusoto_s3::{
    GetObjectRequest,
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
    PluginType,
    TearDownPluginRequest,
    TearDownPluginRequestProto,
    TearDownPluginResponse,
    TearDownPluginResponseProto,
};
use tokio::io::AsyncReadExt;
use tonic::{
    transport::Server,
    Request,
    Response,
    Status,
};

use crate::{
    db::client::{
        GetPluginRow,
        PluginRegistryDbClient,
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
            PluginRegistryServiceError::PluginRegistryDeserializationError(_) => {
                Status::invalid_argument("Unable to deserialize message")
            }
            PluginRegistryServiceError::NomadClientError(_) => {
                Status::internal("Failed RPC with Nomad")
            }
            PluginRegistryServiceError::NomadCliError(_) => {
                Status::internal("Failed using Nomad CLI")
            }
        }
    }
}

use std::net::SocketAddr;

use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct PluginRegistryServiceConfig {
    #[structopt(env)]
    plugin_s3_bucket_aws_account_id: String,
    #[structopt(env)]
    plugin_s3_bucket_name: String,
    #[structopt(env)]
    plugin_registry_bind_address: SocketAddr,
    #[structopt(env)]
    plugin_registry_db_hostname: String,
    #[structopt(env)]
    plugin_registry_db_port: u16,
    #[structopt(env)]
    plugin_registry_db_username: String,
    #[structopt(env)]
    plugin_registry_db_password: String,
}

pub struct PluginRegistry {
    db_client: PluginRegistryDbClient,
    nomad_client: NomadClient,
    nomad_cli: NomadCli,
    s3: S3Client,
    plugin_bucket_name: String,
    plugin_bucket_owner_id: String,
}

impl PluginRegistry {
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
                bucket: self.plugin_bucket_name.clone(),
                key: s3_key.clone(),
                expected_bucket_owner: Some(self.plugin_bucket_owner_id.clone()),
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
        let GetPluginRow {
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

    #[tracing::instrument(skip(self, request), err)]
    async fn deploy_plugin(
        &self,
        request: DeployPluginRequest,
    ) -> Result<DeployPluginResponse, PluginRegistryServiceError> {
        let plugin_id = request.plugin_id;
        let plugin_row = self.db_client.get_plugin(&plugin_id).await?;

        deploy_plugin::deploy_plugin(
            &self.nomad_client,
            &self.nomad_cli,
            plugin_row,
            &self.plugin_bucket_owner_id,
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
        request: Request<DeployPluginRequestProto>,
    ) -> Result<Response<DeployPluginResponseProto>, Status> {
        let request: DeployPluginRequestProto = request.into_inner();
        let request =
            DeployPluginRequest::try_from(request).map_err(PluginRegistryServiceError::from)?;

        let response = self.deploy_plugin(request).await?;
        Ok(Response::new(response.into()))
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
        db_client: PluginRegistryDbClient::new(&postgres_address).await?,
        nomad_client: NomadClient::from_env(),
        nomad_cli: NomadCli::default(),
        s3: S3Client::from_env(),
        plugin_bucket_name: service_config.plugin_s3_bucket_name,
        plugin_bucket_owner_id: service_config.plugin_s3_bucket_aws_account_id,
    };

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
