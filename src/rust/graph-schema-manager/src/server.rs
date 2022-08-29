use rust_proto::{
    graplinc::grapl::{
        api::graph_schema_manager::v1beta1::{
            messages::{
                DeploySchemaRequest,
                DeploySchemaResponse,
                GetEdgeSchemaRequest,
                GetEdgeSchemaResponse,
                SchemaType,
            },
            server::GraphSchemaManagerApi,
        },
        common::v1beta1::types::EdgeName,
    },
    protocol::status::Status,
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
    #[error("NonUtf8 GraphQL Schema {0}")]
    NonUtf8GraphQLSchema(std::string::FromUtf8Error),
    #[error("DeployGraphqlError {0}")]
    DeployGraphqlError(#[from] DeployGraphqlError),
    #[error("GetEdgeSchema sqlx error {0}")]
    GetEdgeSchemaSqlxError(sqlx::Error),
    #[error("Invalid ReverseEdgeName: {0}")]
    InvalidReverseEdgeName(SerDeError),
}

impl From<GraphSchemaManagerServiceError> for Status {
    fn from(error: GraphSchemaManagerServiceError) -> Self {
        match error {
            GraphSchemaManagerServiceError::NonUtf8GraphQLSchema(e) => {
                Status::invalid_argument(format!("NonUtf8GraphQLSchema - {}", e))
            }
            GraphSchemaManagerServiceError::DeployGraphqlError(DeployGraphqlError::SqlxError(
                e,
            )) => Status::internal(format!("SqlError during deployment - {}", e)),
            GraphSchemaManagerServiceError::DeployGraphqlError(e) => {
                Status::invalid_argument(format!("DeployGraphqlError - {}", e))
            }
            GraphSchemaManagerServiceError::GetEdgeSchemaSqlxError(e) => {
                Status::internal(format!("SqlError during deployment - {}", e))
            }
            GraphSchemaManagerServiceError::InvalidReverseEdgeName(name) => {
                Status::internal(format!("InvalidReverseEdgeName - {}", name))
            }
        }
    }
}

pub struct GraphSchemaManager {
    pub db_client: SchemaDbClient,
}

#[async_trait::async_trait]
impl GraphSchemaManagerApi for GraphSchemaManager {
    type Error = GraphSchemaManagerServiceError;

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
            .get_edge_schema(tenant_id, node_type, edge_name)
            .await
            .map_err(GraphSchemaManagerServiceError::GetEdgeSchemaSqlxError)?;

        Ok(GetEdgeSchemaResponse {
            reverse_edge_name: EdgeName::try_from(response.reverse_edge_name)
                .map_err(GraphSchemaManagerServiceError::InvalidReverseEdgeName)?,
            cardinality: response.forward_edge_cardinality.into(),
            reverse_cardinality: response.reverse_edge_cardinality.into(),
        })
    }
}
