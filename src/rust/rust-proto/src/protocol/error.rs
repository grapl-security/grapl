use thiserror::Error;

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum ServeError {
    #[error("encountered tonic error {0}")]
    TransportError(#[from] tonic::transport::Error),
}