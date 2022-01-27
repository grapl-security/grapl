use grapl_utils::future_ext::GraplFutureExt;
use rust_proto::plugin_registry::CreatePluginRequest;

#[derive(sqlx::FromRow)]
pub struct GetPluginRow {
    pub plugin_id: uuid::Uuid,
    pub tenant_id: uuid::Uuid,
    pub display_name: String,
    pub plugin_type: String,
    pub artifact_s3_key: String,
}

pub struct PluginRegistryDbClient {
    pool: sqlx::PgPool,
}

impl PluginRegistryDbClient {
    pub async fn get_plugin(&self, plugin_id: &uuid::Uuid) -> Result<GetPluginRow, sqlx::Error> {
        sqlx::query_as(
            r"
        SELECT
        plugin_id,
        tenant_id,
        display_name,
        plugin_type,
        artifact_s3_key
        FROM plugins
        WHERE plugin_id = $0
            ",
        )
        .bind(&plugin_id)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn create_plugin(
        &self,
        plugin_id: &uuid::Uuid,
        request: &CreatePluginRequest,
        s3_key: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
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
        )
        .bind(plugin_id)
        .bind(&request.plugin_type.type_name())
        .bind(&request.display_name)
        .bind(&request.tenant_id)
        .bind(s3_key)
        .execute(&self.pool)
        .await
        .map(|_| ())
    }

    pub async fn new(postgres_address: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let client = Self {
            pool: sqlx::PgPool::connect(&postgres_address)
                .timeout(std::time::Duration::from_secs(5))
                .await??,
        };

        sqlx::migrate!().run(&client.pool).await?;

        Ok(client)
    }
}
