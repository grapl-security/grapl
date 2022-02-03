use grapl_utils::future_ext::GraplFutureExt;
use rust_proto::plugin_registry::CreatePluginRequest;
use sqlx::types::chrono::{
    DateTime,
    Utc,
};

#[derive(sqlx::FromRow)]
pub struct PluginRow {
    pub plugin_id: uuid::Uuid,
    pub tenant_id: uuid::Uuid,
    pub display_name: String,
    pub plugin_type: String,
    pub artifact_s3_key: String,
}

#[derive(Clone, Debug, Eq, PartialEq, sqlx::Type)]
#[sqlx(type_name = "plugin_deployment_status", rename_all = "lowercase")]
pub enum PluginDeploymentStatus {
    Fail,
    Success,
}

impl<T, E> From<&Result<T, E>> for PluginDeploymentStatus {
    fn from(res: &Result<T, E>) -> Self {
        match res.as_ref() {
            Ok(_) => PluginDeploymentStatus::Success,
            Err(_) => PluginDeploymentStatus::Fail,
        }
    }
}

#[derive(sqlx::FromRow)]
pub struct PluginDeployment {
    pub id: i32,
    pub plugin_id: uuid::Uuid,
    pub deploy_time: DateTime<Utc>,
    pub status: PluginDeploymentStatus,
}

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
