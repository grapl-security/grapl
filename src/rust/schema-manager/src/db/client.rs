use grapl_config::PostgresClient;
use rust_proto::graplinc::grapl::common::v1beta1::types::{
    EdgeName,
    NodeType,
};

use crate::{
    config::SchemaDbConfig,
    db::models::{
        GetEdgeSchemaRequestRow,
        StoredEdgeCardinality,
    },
    server::SchemaManagerServiceError,
};

pub struct SchemaDbClient {
    pub pool: sqlx::PgPool, // TODO depublicize
}

#[async_trait::async_trait]
impl PostgresClient for SchemaDbClient {
    type Config = SchemaDbConfig;
    type Error = grapl_config::PostgresDbInitError;

    fn new(pool: sqlx::Pool<sqlx::Postgres>) -> Self {
        Self { pool }
    }

    #[tracing::instrument]
    async fn migrate(pool: &sqlx::Pool<sqlx::Postgres>) -> Result<(), sqlx::migrate::MigrateError> {
        tracing::info!(message = "Performing database migration");

        sqlx::migrate!().run(pool).await
    }
}

impl SchemaDbClient {
    pub async fn get_edge_schema(
        &self,
        tenant_id: uuid::Uuid,
        node_type: NodeType,
        edge_name: EdgeName,
    ) -> Result<GetEdgeSchemaRequestRow, SchemaManagerServiceError> {
        sqlx::query_as!(
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
            node_type.value,
            edge_name.value,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(SchemaManagerServiceError::GetEdgeSchemaSqlxError)
    }
}
