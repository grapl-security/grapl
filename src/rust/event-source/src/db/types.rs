use sqlx::types::chrono::{
    DateTime,
    Utc,
};
use uuid::Uuid;

pub struct EventSourceRow {
    pub event_source_id: Uuid,
    pub tenant_id: Uuid,
    pub display_name: String,
    pub description: String,
    pub created_time: DateTime<Utc>,
    pub last_updated_time: DateTime<Utc>,
    pub active: bool,
}
