use rust_proto::protocol::status::Status;

#[derive(thiserror::Error, Debug)]
pub enum EventSourceError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    DbInit(#[from] grapl_config::PostgresDbInitError),
}

impl From<EventSourceError> for Status {
    fn from(e: EventSourceError) -> Self {
        Status::internal(e.to_string())
    }
}
