use grapl_config::PostgresClient;
use rust_proto::graplinc::grapl::common::v1beta1::types::{
    EdgeName, NodeType,
};
use sqlx::{Transaction, Postgres};

use crate::{
    config::SchemaDbConfig,
    db::models::{
        GetEdgeSchemaRequestRow,
        StoredEdgeCardinality,
    },
};

use grapl_graphql_codegen::{
    edge::Edge as CodegenEdge, 
    node_type::NodeType as CodegenNodeType,
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
    ) -> Result<GetEdgeSchemaRequestRow, sqlx::Error> {
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
    }

    pub async fn insert_edge_schema(
        &self,
        tenant_id: uuid::Uuid,
        node_type: &CodegenNodeType,
        edge: &CodegenEdge,
        schema_version: u32,
        txn: &mut Transaction<'_, Postgres>,
    ) -> Result<(), sqlx::Error> {
        let forward_edge_cardinality = if edge.relationship.to_one() {
            StoredEdgeCardinality::ToOne
        } else {
            StoredEdgeCardinality::ToMany
        };

        let reverse_edge_cardinality = if edge.relationship.reverse().to_one() {
            StoredEdgeCardinality::ToOne
        } else {
            StoredEdgeCardinality::ToMany
        };

        sqlx::query!(
            r#"
            INSERT INTO schema_manager.edge_schemas (
                tenant_id,
                node_type,
                schema_version,
                forward_edge_name,
                reverse_edge_name,
                forward_edge_cardinality,
                reverse_edge_cardinality
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
            tenant_id,
            &node_type.type_name,
            schema_version as i16,
            edge.edge_name,
            edge.reverse_edge_name,
            forward_edge_cardinality as StoredEdgeCardinality,
            reverse_edge_cardinality as StoredEdgeCardinality,
        )
        .execute(&mut *txn)
        .await?;

        sqlx::query!(
            r#"
            INSERT INTO schema_manager.edge_schemas (
                tenant_id,
                node_type,
                schema_version,
                forward_edge_name,
                reverse_edge_name,
                forward_edge_cardinality,
                reverse_edge_cardinality
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
            tenant_id,
            &node_type.type_name,
            schema_version as i16,
            edge.reverse_edge_name,
            edge.edge_name,
            reverse_edge_cardinality as StoredEdgeCardinality,
            forward_edge_cardinality as StoredEdgeCardinality,
        )
        .execute(&mut *txn)
        .await?;

        Ok(())
    }
}
