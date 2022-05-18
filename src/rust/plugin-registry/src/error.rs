use rusoto_s3::{
    GetObjectError,
    PutObjectError,
};
use rust_proto_new::{
    protocol::status::Status,
    SerDeError,
};

use crate::{
    db::serde::DatabaseSerDeError,
    nomad,
};

#[derive(Debug, thiserror::Error)]
pub enum PluginRegistryServiceError {
    #[error(transparent)]
    SqlxError(#[from] sqlx::Error),
    #[error(transparent)]
    S3PutObjectError(#[from] rusoto_core::RusotoError<PutObjectError>),
    #[error(transparent)]
    S3GetObjectError(#[from] rusoto_core::RusotoError<GetObjectError>),
    #[error("EmptyObject")]
    EmptyObject,
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    SerDeError(#[from] SerDeError),
    #[error(transparent)]
    DatabaseSerDeError(#[from] DatabaseSerDeError),
    #[error(transparent)]
    NomadClientError(#[from] nomad::client::NomadClientError),
    #[error(transparent)]
    NomadCliError(#[from] nomad::cli::NomadCliError),
    #[error("NomadJobAllocationError")]
    NomadJobAllocationError,
    #[error(transparent)]
    StreamError(#[from] Status),
    #[error("StreamInputError {0}")]
    StreamInputError(&'static str),
    // TODO: These errs are meant to be human-readable and are not directly
    // sent over the wire, so add {0}s to them!
}

impl From<PluginRegistryServiceError> for Status {
    /**
     * Convert useful internal errors into tonic::Status that can be
     * safely sent over the wire. (Don't include any specific IDs etc)
     */
    fn from(err: PluginRegistryServiceError) -> Self {
        type Error = PluginRegistryServiceError;
        match err {
            Error::SqlxError(sqlx::Error::Configuration(_)) => {
                Status::internal("Invalid SQL configuration")
            }
            Error::SqlxError(_) => Status::internal("Failed to operate on postgres"),
            Error::S3PutObjectError(_) => Status::internal("Failed to put s3 object"),
            Error::S3GetObjectError(_) => Status::internal("Failed to get s3 object"),
            Error::EmptyObject => Status::internal("S3 Object was unexpectedly empty"),
            Error::IoError(_) => Status::internal("IoError"),
            Error::SerDeError(_) => Status::invalid_argument("Unable to deserialize message"),
            Error::DatabaseSerDeError(_) => {
                Status::invalid_argument("Unable to deserialize message from database")
            }
            Error::NomadClientError(_) => Status::internal("Failed RPC with Nomad"),
            Error::NomadCliError(_) => Status::internal("Failed using Nomad CLI"),
            Error::NomadJobAllocationError => {
                Status::internal("Unable to allocate Nomad job - it may be out of resources.")
            }
            Error::StreamError(_) => Status::internal("Unexpected error in Stream RPC"),
            Error::StreamInputError(_) => {
                Status::invalid_argument("Unexpected input to Stream RPC")
            }
        }
    }
}
