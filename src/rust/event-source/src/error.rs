use rust_proto_new::protocol::status::Status;

use crate::db::EventSourceDbInitError;

/// An example, silly error class
#[derive(thiserror::Error, Debug)]
pub enum EventSourceError {
    #[error("DbInitError")]
    DbInitError(#[from] EventSourceDbInitError),
}

impl From<EventSourceError> for Status {
    fn from(e: EventSourceError) -> Self {
        Status::internal(e.to_string())
    }
}
