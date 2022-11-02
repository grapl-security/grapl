use rust_proto::{
    graplinc::grapl::{
        api::{
            graph_schema_manager::v1beta1::{
                messages::{
                    DeploySchemaRequest,
                    DeploySchemaResponse,
                    GetEdgeSchemaRequest,
                    GetEdgeSchemaResponse,
                    SchemaType,
                },
                server::GraphSchemaManagerApi,
            },
            protocol::status::Status,
        },
        common::v1beta1::types::EdgeName,
    },
    SerDeError,
};

use crate::{
    db::client::SchemaDbClient,
    deploy_graphql_schema::{
        deploy_graphql_schema,
        DeployGraphqlError,
    },
};

#[derive(thiserror::Error, Debug)]
pub enum GraphSchemaManagerServiceError {
    #[error("NonUtf8 GraphQL Schema: '{0}'")]
    NonUtf8GraphQLSchema(std::string::FromUtf8Error),
    #[error("DeployGraphqlError: '{0}'")]
    DeployGraphqlError(#[from] DeployGraphqlError),
    #[error("GetEdgeSchema sqlx error: '{0}'")]
    GetEdgeSchemaSqlxError(sqlx::Error),
    #[error("Invalid ReverseEdgeName: '{0}'")]
    InvalidReverseEdgeName(SerDeError),
    #[error("GetEdgeSchema: Edge not found for tenant_id={tenant_id}, node_type={node_type}, edge_name={edge_name}")]
    EdgeSchemaNotFound {
        tenant_id: uuid::Uuid,
        node_type: String,
        edge_name: String,
    },
}

impl From<GraphSchemaManagerServiceError> for Status {
    fn from(error: GraphSchemaManagerServiceError) -> Self {
        let msg = error.to_string();
        match error {
            GraphSchemaManagerServiceError::NonUtf8GraphQLSchema(_) => {
                Status::invalid_argument(msg)
            }
            GraphSchemaManagerServiceError::DeployGraphqlError(DeployGraphqlError::SqlxError(
                _,
            )) => Status::internal(msg),
            GraphSchemaManagerServiceError::DeployGraphqlError(_) => Status::invalid_argument(msg),
            GraphSchemaManagerServiceError::EdgeSchemaNotFound { .. } => Status::internal(msg),
            GraphSchemaManagerServiceError::GetEdgeSchemaSqlxError(_) => Status::internal(msg),
            GraphSchemaManagerServiceError::InvalidReverseEdgeName(_) => Status::internal(msg),
        }
    }
}

pub struct GraphSchemaManager {
    pub db_client: SchemaDbClient,
}

#[async_trait::async_trait]
impl GraphSchemaManagerApi for GraphSchemaManager {
    type Error = GraphSchemaManagerServiceError;

    #[tracing::instrument(skip(self, request), err)]
    async fn deploy_schema(
        &self,
        request: DeploySchemaRequest,
    ) -> Result<DeploySchemaResponse, Self::Error> {
        match request.schema_type {
            SchemaType::GraphqlV0 => {
                let schema = String::from_utf8(request.schema.to_vec())
                    .map_err(GraphSchemaManagerServiceError::NonUtf8GraphQLSchema)?;

                deploy_graphql_schema(
                    request.tenant_id,
                    &schema,
                    request.schema_version,
                    &self.db_client,
                )
                .await?;
                Ok(DeploySchemaResponse {})
            }
        }
    }

    #[tracing::instrument(skip(self), err)]
    async fn get_edge_schema(
        &self,
        request: GetEdgeSchemaRequest,
    ) -> Result<GetEdgeSchemaResponse, Self::Error> {
        let GetEdgeSchemaRequest {
            tenant_id,
            node_type,
            edge_name,
        } = request;

        let response = self
            .db_client
            .get_edge_schema(tenant_id, node_type.clone(), edge_name.clone())
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => Self::Error::EdgeSchemaNotFound {
                    tenant_id,
                    node_type: node_type.value,
                    edge_name: edge_name.value,
                },
                _ => Self::Error::GetEdgeSchemaSqlxError(e),
            })?;

        Ok(GetEdgeSchemaResponse {
            reverse_edge_name: EdgeName::try_from(response.reverse_edge_name)
                .map_err(GraphSchemaManagerServiceError::InvalidReverseEdgeName)?,
            cardinality: response.forward_edge_cardinality.into(),
            reverse_cardinality: response.reverse_edge_cardinality.into(),
        })
    }
}
