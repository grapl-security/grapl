use crate::{
    graplinc::grapl::api::schema_manager::v1beta1::messages::{
        DeployModelRequest,
        DeployModelResponse,
        GetEdgeSchemaRequest,
        GetEdgeSchemaResponse,
    },
    protobufs::graplinc::grapl::api::schema_manager::v1beta1::{
        schema_manager_service_client::SchemaManagerServiceClient as SchemaManagerServiceClientProto,
        DeployModelRequest as DeployModelRequestProto,
        GetEdgeSchemaRequest as GetEdgeSchemaRequestProto,
    },
    protocol::status::Status,
    SerDeError,
};

#[derive(thiserror::Error, Debug)]
pub enum SchemaManagerClientError {
    #[error("Failed to deserialize response {0}")]
    SerDeError(#[from] SerDeError),
    #[error("Status {0}")]
    Status(Status),
    #[error("ConnectError")]
    ConnectError(tonic::transport::Error),
}

#[derive(Clone)]
pub struct SchemaManagerClient {
    inner: SchemaManagerServiceClientProto<tonic::transport::Channel>,
}

impl SchemaManagerClient {
    pub async fn connect<T>(endpoint: T) -> Result<Self, SchemaManagerClientError>
    where
        T: TryInto<tonic::transport::Endpoint>,
        T::Error: std::error::Error + Send + Sync + 'static,
    {
        Ok(SchemaManagerClient {
            inner: SchemaManagerServiceClientProto::connect(endpoint)
                .await
                .map_err(SchemaManagerClientError::ConnectError)?,
        })
    }

    pub async fn deploy_model(
        &mut self,
        request: DeployModelRequest,
    ) -> Result<DeployModelResponse, SchemaManagerClientError> {
        let raw_request: DeployModelRequestProto = request.into();
        let raw_response = self
            .inner
            .deploy_model(raw_request)
            .await
            .map_err(|s| SchemaManagerClientError::Status(s.into()))?;
        let proto_response = raw_response.into_inner();
        let response = proto_response.try_into()?;
        Ok(response)
    }

    pub async fn get_edge_schema(
        &mut self,
        request: GetEdgeSchemaRequest,
    ) -> Result<GetEdgeSchemaResponse, SchemaManagerClientError> {
        let raw_request: GetEdgeSchemaRequestProto = request.into();
        let raw_response = self
            .inner
            .get_edge_schema(raw_request)
            .await
            .map_err(|s| SchemaManagerClientError::Status(s.into()))?;
        let proto_response = raw_response.into_inner();
        let response = proto_response.try_into()?;
        Ok(response)
    }
}
