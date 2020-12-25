use thiserror::Error;

pub enum Recoverable {
    Transient,
    Persistent,
}

pub trait CheckedError: std::error::Error {
    fn error_type(&self) -> Recoverable;
}


#[derive(Error, Debug)]
pub enum ExecutorError<
    CacheErrorT,
    HandlerErrorT,
>
    where
        CacheErrorT: CheckedError,
        HandlerErrorT: CheckedError
{
    #[error("Failed to receive sqs messages")]
    SqsReceiveError(#[from] rusoto_core::RusotoError<rusoto_sqs::ReceiveMessageError>),
    #[error("Cache error")]
    CacheError(CacheErrorT),
    #[error("Handler error")]
    HandlerError(HandlerErrorT)
}

impl<CacheErrorT, HandlerErrorT,> CheckedError for ExecutorError<
    CacheErrorT,
    HandlerErrorT,
>
    where
        CacheErrorT: CheckedError,
        HandlerErrorT: CheckedError
{
    fn error_type(&self) -> Recoverable {
        match self {
            Self::SqsReceiveError(e) => {
                Recoverable::Transient
            }
            Self::CacheError(e) => e.error_type(),
            Self::HandlerError(e) => e.error_type(),
        }
    }
}