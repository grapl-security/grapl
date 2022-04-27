use grapl_utils::future_ext::GraplFutureExt;
use rust_proto_new::graplinc::grapl::api::plugin_registry::v1beta1::CreatePluginRequest;

use super::models::{
    PluginDeploymentRow,
    PluginDeploymentStatus,
    PluginRow,
};

pub struct PluginRegistryDbClient {
    pool: sqlx::PgPool,
}

impl PluginRegistryDbClient {
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
            artifact_s3_key
            FROM plugins
            WHERE plugin_id = $1
            ",
            plugin_id
        )
        .fetch_one(&self.pool)
        .await
    }

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
                deploy_time,
                status AS "status: PluginDeploymentStatus"
            FROM plugin_deployment
            WHERE plugin_id = $1
            ORDER BY id desc limit 1;
            "#,
            plugin_id
        )
        .fetch_one(&self.pool)
        .await
    }

    #[tracing::instrument(skip(self, request, s3_key), err)]
    pub async fn create_plugin(
        &self,
        plugin_id: &uuid::Uuid,
        request: &CreatePluginRequest,
        s3_key: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
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
            &request.plugin_type.type_name(),
            &request.display_name,
            &request.tenant_id,
            s3_key,
        )
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

    pub async fn new(postgres_address: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let client = Self {
            pool: sqlx::PgPool::connect(postgres_address)
                .timeout(std::time::Duration::from_secs(5))
                .await??,
        };

        sqlx::migrate!().run(&client.pool).await?;

        Ok(client)
    }
}
