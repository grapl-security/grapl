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

/// The whole PluginDeployment table is currently just appended to, not
/// consumed anywhere.
#[derive(sqlx::FromRow)]
pub struct PluginDeploymentRow {
    pub id: i32,
    pub plugin_id: uuid::Uuid,
    pub deploy_time: DateTime<Utc>,
    pub status: PluginDeploymentStatus,
}
