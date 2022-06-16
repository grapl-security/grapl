use sqlx::types::time::PrimitiveDateTime;
use uuid::Uuid;

pub struct EventSourceRow {
    pub event_source_id: Uuid,
    pub tenant_id: Uuid,
    pub display_name: String,
    pub description: String,
    pub created_time: PrimitiveDateTime,
    pub last_updated_time: PrimitiveDateTime,
    pub active: bool,
}
