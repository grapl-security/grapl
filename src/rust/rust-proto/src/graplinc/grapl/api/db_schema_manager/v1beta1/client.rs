use crate::{
    graplinc::grapl::api::db_schema_manager::v1beta1::messages::{
        DeployGraphSchemasRequest,
        DeployGraphSchemasResponse,
    },
    protobufs::graplinc::grapl::api::db_schema_manager::v1beta1::{
        db_schema_manager_service_client::DbSchemaManagerServiceClient,
        DeployGraphSchemasRequest as DeployGraphSchemasRequestProto,
    },
    protocol::status::Status,
    SerDeError,
};

#[derive(thiserror::Error, Debug)]
pub enum DbSchemaManagerClientError {
    #[error("Failed to deserialize response {0}")]
    SerDeError(#[from] SerDeError),
    #[error("Status {0}")]
    Status(#[from] Status),
    #[error("ConnectError")]
    ConnectError(tonic::transport::Error),
}

#[derive(Clone)]
pub struct DbSchemaManagerClient {
    inner: DbSchemaManagerServiceClient<tonic::transport::Channel>,
}

impl DbSchemaManagerClient {
    pub async fn connect<T>(endpoint: T) -> Result<Self, DbSchemaManagerClientError>
    where
        T: TryInto<tonic::transport::Endpoint>,
        T::Error: std::error::Error + Send + Sync + 'static,
    {
        Ok(DbSchemaManagerClient {
            inner: DbSchemaManagerServiceClient::connect(endpoint)
                .await
                .map_err(DbSchemaManagerClientError::ConnectError)?,
        })
    }

    pub async fn query_graph_with_uid(
        &mut self,
        request: DeployGraphSchemasRequest,
    ) -> Result<DeployGraphSchemasResponse, DbSchemaManagerClientError> {
        let request: DeployGraphSchemasRequestProto = request.into();
        Ok(self
            .inner
            .deploy_graph_schemas(request)
            .await
            .map_err(Status::from)?
            .into_inner()
            .try_into()?)
    }
}
