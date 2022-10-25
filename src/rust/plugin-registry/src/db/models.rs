use rust_proto::graplinc::grapl::api::plugin_registry::v1beta1;
use sqlx::types::chrono::{
    DateTime,
    Utc,
};

#[derive(sqlx::FromRow)]
pub struct PluginIdRow {
    pub plugin_id: uuid::Uuid,
}

#[derive(sqlx::FromRow)]
pub struct PluginRow {
    pub plugin_id: uuid::Uuid,
    pub tenant_id: uuid::Uuid,
    pub display_name: String,
    pub plugin_type: String,
    pub artifact_s3_key: String,
    pub event_source_id: Option<uuid::Uuid>,
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

impl From<PluginDeploymentStatus> for v1beta1::PluginDeploymentStatus {
    fn from(plugin_deployment_status: PluginDeploymentStatus) -> Self {
        match plugin_deployment_status {
            PluginDeploymentStatus::Fail => v1beta1::PluginDeploymentStatus::Fail,
            PluginDeploymentStatus::Success => v1beta1::PluginDeploymentStatus::Success,
        }
    }
}

/// The whole PluginDeployment table is currently just appended to, not
/// consumed anywhere.
#[derive(sqlx::FromRow)]
pub struct PluginDeploymentRow {
    pub id: i64,
    pub plugin_id: uuid::Uuid,
    pub timestamp: DateTime<Utc>,
    pub status: PluginDeploymentStatus,
    pub deployed: bool,
}
