use rust_proto_new::protocol::status::Status;

use crate::db::EventSourceDbError;

#[derive(thiserror::Error, Debug)]
pub enum EventSourceError {
    #[error("DbError")]
    DbError(#[from] EventSourceDbError),
}

impl From<EventSourceError> for Status {
    fn from(e: EventSourceError) -> Self {
        Status::internal(e.to_string())
    }
}
