use grapl_config::PostgresClient;
use rust_proto::graplinc::grapl::api::plugin_registry::v1beta1::PluginType;

use super::models::{
    PluginDeploymentRow,
    PluginDeploymentStatus,
    PluginIdRow,
    PluginRow,
};
use crate::server::service::PluginRegistryDbConfig;

pub struct PluginRegistryDbClient {
    pool: sqlx::PgPool,
}

#[async_trait::async_trait]
impl PostgresClient for PluginRegistryDbClient {
    type Config = PluginRegistryDbConfig;
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

pub struct DbCreatePluginArgs {
    pub tenant_id: uuid::Uuid,
    pub display_name: String,
    pub plugin_type: PluginType,
    pub event_source_id: Option<uuid::Uuid>,
}

impl PluginRegistryDbClient {
    #[tracing::instrument(skip(self), err)]
    pub async fn get_analyzers_for_tenant(
        &self,
        tenant_id: &uuid::Uuid,
    ) -> Result<Vec<PluginIdRow>, sqlx::Error> {
        sqlx::query_as!(
            PluginIdRow,
            r"
            SELECT
            plugin_id
            FROM plugins
            WHERE tenant_id = $1 AND plugin_type = $2;
            ",
            tenant_id,
            PluginType::Analyzer.type_name(),
        )
        .fetch_all(&self.pool)
        .await
    }

    #[tracing::instrument(skip(self), err)]
    pub async fn get_generators_for_event_source(
        &self,
        event_source_id: &uuid::Uuid,
    ) -> Result<Vec<PluginIdRow>, sqlx::Error> {
        sqlx::query_as!(
            PluginIdRow,
            r"
            SELECT
            plugin_id
            FROM plugins
            WHERE event_source_id = $1 AND plugin_type = $2;
            ",
            event_source_id,
            PluginType::Generator.type_name(),
        )
        .fetch_all(&self.pool)
        .await
    }

    #[tracing::instrument(skip(self), err)]
    pub async fn get_plugin(&self, plugin_id: &uuid::Uuid) -> Result<PluginRow, sqlx::Error> {
        sqlx::query_as!(
            PluginRow,
            r"
            SELECT
            plugin_id,
            tenant_id,
            display_name,
            plugin_type,
            artifact_s3_key,
            event_source_id
            FROM plugins
            WHERE plugin_id = $1
            ",
            plugin_id
        )
        .fetch_one(&self.pool)
        .await
    }

    #[allow(dead_code)]
    #[tracing::instrument(skip(self), err)]
    pub async fn get_plugin_deployment(
        &self,
        plugin_id: &uuid::Uuid,
    ) -> Result<PluginDeploymentRow, sqlx::Error> {
        sqlx::query_as!(
            PluginDeploymentRow,
            r#"
            SELECT
                id,
                plugin_id,
                timestamp,
                status AS "status: PluginDeploymentStatus",
                deployed
            FROM plugin_deployment
            WHERE plugin_id = $1
            ORDER BY id desc limit 1;
            "#,
            plugin_id
        )
        .fetch_one(&self.pool)
        .await
    }

    #[tracing::instrument(skip(self, args, s3_key), err)]
    pub async fn create_plugin(
        &self,
        plugin_id: &uuid::Uuid,
        args: DbCreatePluginArgs,
        s3_key: &str,
    ) -> Result<(), sqlx::Error> {
        match args.event_source_id {
            Some(event_source_id) => sqlx::query!(
                r"
                INSERT INTO plugins (
                    plugin_id,
                    plugin_type,
                    display_name,
                    tenant_id,
                    artifact_s3_key,
                    event_source_id
                )
                VALUES ($1::uuid, $2, $3, $4::uuid, $5, $6::uuid)
                ON CONFLICT DO NOTHING;
                ",
                plugin_id,
                &args.plugin_type.type_name(),
                &args.display_name,
                &args.tenant_id,
                s3_key,
                event_source_id,
            ),
            None => sqlx::query!(
                r"
                INSERT INTO plugins (
                    plugin_id,
                    plugin_type,
                    display_name,
                    tenant_id,
                    artifact_s3_key
                )
                VALUES ($1::uuid, $2, $3, $4::uuid, $5)
                ON CONFLICT DO NOTHING;
                ",
                plugin_id,
                &args.plugin_type.type_name(),
                &args.display_name,
                &args.tenant_id,
                s3_key,
            ),
        }
        .execute(&self.pool)
        .await
        .map(|_| ()) // Toss result
    }

    #[tracing::instrument(skip(self), err)]
    pub async fn create_plugin_deployment(
        &self,
        plugin_id: &uuid::Uuid,
        status: PluginDeploymentStatus,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r"
            INSERT INTO plugin_deployment (
                plugin_id,
                status
            )
            VALUES ($1::uuid, $2);
            ",
            plugin_id,
            status as _,
        )
        .execute(&self.pool)
        .await
        .map(|_| ()) // Toss result
    }

    pub async fn deactivate_plugin_deployment(
        &self,
        plugin_id: &uuid::Uuid,
    ) -> Result<(), sqlx::Error> {
        let plugin_deployment_row = self.get_plugin_deployment(plugin_id).await?;
        sqlx::query!(
            r"
            INSERT INTO plugin_deployment (
                id,
                plugin_id,
                status,
                deployed
            ) VALUES (
                $1,
                $2::uuid,
                $3,
                false
            ) ON CONFLICT DO NOTHING;
            ",
            plugin_deployment_row.id,
            plugin_deployment_row.plugin_id,
            plugin_deployment_row.status as _,
        )
        .execute(&self.pool)
        .await
        .map(|_| ())
    }
}
