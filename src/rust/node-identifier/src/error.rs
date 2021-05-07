use sqs_executor::errors::{
    CheckedError,
    Recoverable,
};

#[derive(thiserror::Error, Debug)]
pub enum NodeIdentifierError {
    #[error("Unexpected error")]
    Unexpected,
}

impl CheckedError for NodeIdentifierError {
    fn error_type(&self) -> Recoverable {
        Recoverable::Transient
    }
}
