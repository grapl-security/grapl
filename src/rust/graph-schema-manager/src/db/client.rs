use grapl_config::PostgresClient;
use rust_proto::graplinc::grapl::common::v1beta1::types::{
    EdgeName,
    NodeType,
};
use sqlx::{
    Postgres,
    Transaction,
};

use super::models::StoredPropertyType;
use crate::{
    config::SchemaDbConfig,
    db::models::{
        GetEdgeSchemaRequestRow,
        StoredEdgeCardinality,
    },
};

pub struct SchemaDbClient {
    pool: sqlx::PgPool,
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

pub type Txn<'a> = Transaction<'a, Postgres>;

impl SchemaDbClient {
    pub async fn begin_txn(&self) -> Result<Txn<'_>, sqlx::Error> {
        self.pool.begin().await
    }

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
             FROM graph_schema_manager.edge_schemas
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

    pub async fn insert_node_identity_algorithm(
        &self,
        txn: &mut Txn<'_>,
        tenant_id: uuid::Uuid,
        identity_algorithm: &str,
        node_type_name: &str,
        schema_version: u32,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO graph_schema_manager.node_identity_algorithm (
                tenant_id,
                identity_algorithm,
                node_type,
                schema_version
            )
            VALUES ($1, $2, $3, $4)
            "#,
            tenant_id,
            identity_algorithm,
            node_type_name,
            schema_version as i16,
        )
        .execute(&mut *txn)
        .await?;
        Ok(())
    }

    pub async fn insert_static_identity_args(
        &self,
        txn: &mut Txn<'_>,
        tenant_id: uuid::Uuid,
        node_type_name: &str,
        schema_version: u32,
        static_keys: Vec<String>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO graph_schema_manager.static_identity_arguments (
                tenant_id,
                identity_algorithm,
                node_type,
                schema_version,
                static_key_properties
            )
            VALUES ($1, $2, $3, $4, $5)
            "#,
            tenant_id,
            "static",
            node_type_name,
            schema_version as i16,
            &static_keys[..],
        )
        .execute(&mut *txn)
        .await?;
        Ok(())
    }

    pub async fn insert_session_identity_args(
        &self,
        txn: &mut Txn<'_>,
        tenant_id: uuid::Uuid,
        node_type_name: &str,
        schema_version: u32,
        pseudo_keys: Vec<String>,
        creation_timestamp_property: &str,
        last_seen_timestamp_property: &str,
        termination_timestamp_property: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO graph_schema_manager.session_identity_arguments (
                tenant_id,
                identity_algorithm,
                node_type,
                schema_version,
                pseudo_key_properties,
                negation_key_properties,
                creation_timestamp_property,
                last_seen_timestamp_property,
                termination_timestamp_property
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
            tenant_id,
            "session",
            node_type_name,
            schema_version as i16,
            &pseudo_keys[..],
            &[][..], // todo: negation keys are not supported in the parser
            creation_timestamp_property,
            last_seen_timestamp_property,
            termination_timestamp_property
        )
        .execute(&mut *txn)
        .await?;

        Ok(())
    }

    pub async fn insert_node_property(
        &self,
        txn: &mut Txn<'_>,
        tenant_id: uuid::Uuid,
        node_type_name: &str,
        schema_version: u32,
        property_predicate_name: &str,
        predicate_type_name: StoredPropertyType,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO graph_schema_manager.property_schemas (
                tenant_id,
                node_type,
                schema_version,
                property_name,
                property_type,
                identity_only
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
            tenant_id,
            node_type_name,
            schema_version as i16,
            property_predicate_name,
            predicate_type_name as StoredPropertyType,
            false, // todo: implement identification only properties
        )
        .execute(&mut *txn)
        .await?;

        Ok(())
    }

    pub async fn insert_node_schema(
        &self,
        txn: &mut Txn<'_>,
        tenant_id: uuid::Uuid,
        identity_algorithm: &str,
        node_type_name: &str,
        schema_version: u32,
        raw_schema: &str,
        schema_type: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO graph_schema_manager.node_schemas (
                tenant_id,
                identity_algorithm,
                node_type,
                schema_version,
                raw_schema,
                schema_type
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
            tenant_id,
            identity_algorithm,
            node_type_name,
            schema_version as i16,
            raw_schema.as_bytes(),
            schema_type,
        )
        .execute(&mut *txn)
        .await?;
        Ok(())
    }

    pub async fn insert_edge_schema(
        &self,
        txn: &mut Txn<'_>,
        tenant_id: uuid::Uuid,
        node_type_name: &str,
        forward_edge_name: &str,
        forward_edge_cardinality: StoredEdgeCardinality,
        reverse_edge_name: &str,
        reverse_edge_cardinality: StoredEdgeCardinality,
        schema_version: u32,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO graph_schema_manager.edge_schemas (
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
            node_type_name,
            schema_version as i16,
            forward_edge_name,
            reverse_edge_name,
            forward_edge_cardinality as StoredEdgeCardinality,
            reverse_edge_cardinality as StoredEdgeCardinality,
        )
        .execute(&mut *txn)
        .await?;

        sqlx::query!(
            r#"
            INSERT INTO graph_schema_manager.edge_schemas (
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
            node_type_name,
            schema_version as i16,
            reverse_edge_name,
            forward_edge_name,
            reverse_edge_cardinality as StoredEdgeCardinality,
            forward_edge_cardinality as StoredEdgeCardinality,
        )
        .execute(&mut *txn)
        .await?;

        Ok(())
    }
}
