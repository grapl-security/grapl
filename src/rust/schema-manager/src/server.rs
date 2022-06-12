use sqlx::PgPool;
use rust_proto_new::graplinc::grapl::api::schema_manager::v1beta1::messages::{DeployModelRequest, DeployModelResponse, EdgeCardinality, GetEdgeSchemaRequest, GetEdgeSchemaResponse, SchemaType};
use rust_proto_new::graplinc::grapl::api::schema_manager::v1beta1::server::SchemaManagerApi;
use rust_proto_new::protocol::status::Status;
use crate::StoredEdgeCardinality;

#[derive(thiserror::Error, Debug)]
pub enum SchemaManagerServiceError {
    #[error("NonUtf8 GraphQL Schema {0}")]
    NonUtf8GraphQLSchema(std::string::FromUtf8Error),
    #[error("DeployGraphqlError {0}")]
    DeployGraphqlError(#[from] crate::DeployGraphqlError),
    #[error("GetEdgeSchema sqlx error {0}")]
    GetEdgeSchemaSqlxError(sqlx::Error),
}

impl From<SchemaManagerServiceError> for Status {
    fn from(error: SchemaManagerServiceError) -> Self {
        unimplemented!()
    }
}

pub struct SchemaManager {
    pool: PgPool,
}

#[async_trait::async_trait]
impl SchemaManagerApi for SchemaManager {
    type Error = SchemaManagerServiceError;

    async fn deploy_model(
        &self,
        request: DeployModelRequest,
    ) -> Result<DeployModelResponse, Self::Error> {
        match request.schema_type {
            SchemaType::GraphqlV0 => {
                let schema = String::from_utf8(request.schema)
                    .map_err(SchemaManagerServiceError::NonUtf8GraphQLSchema)?;

                crate::deploy_graphql_plugin(
                    request.tenant_id,
                    &schema,
                    request.schema_version,
                    &self.pool,
                ).await?;
                Ok(DeployModelResponse {})
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

        let response = sqlx::query_as!(
            GetEdgeSchemaRequestRow,
            r#"select
                reverse_edge_name,
                forward_edge_cardinality as "forward_edge_cardinality: StoredEdgeCardinality",
                reverse_edge_cardinality as "reverse_edge_cardinality: StoredEdgeCardinality"
             FROM schema_manager.edge_schemas
             WHERE
                 tenant_id = $1 AND
                 node_type = $2 AND
                 forward_edge_name = $3
             ORDER BY schema_version DESC
             LIMIT 1;
                 "#,
            tenant_id,
            node_type,
            edge_name,
        ).fetch_one(&self.pool).await
            .map_err(SchemaManagerServiceError::GetEdgeSchemaSqlxError)?;

        Ok(GetEdgeSchemaResponse {
            reverse_edge_name: response.reverse_edge_name,
            cardinality: response.forward_edge_cardinality.into(),
            reverse_cardinality: response.reverse_edge_cardinality.into(),
        })
    }
}

#[derive(sqlx::Type, Clone, Debug)]
struct GetEdgeSchemaRequestRow {
    reverse_edge_name: String,
    forward_edge_cardinality: StoredEdgeCardinality,
    reverse_edge_cardinality: StoredEdgeCardinality,
}

impl From<StoredEdgeCardinality> for EdgeCardinality {
    fn from(c: StoredEdgeCardinality) -> Self {
        match c {
            StoredEdgeCardinality::ToOne => EdgeCardinality::ToOne,
            StoredEdgeCardinality::ToMany => EdgeCardinality::ToMany,
        }
    }
}