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
    #[error("SqlxError")]
    SqlxError(#[from] sqlx::Error),
    #[error("S2PutObjectError")]
    PutObjectError(#[from] rusoto_core::RusotoError<PutObjectError>),
    #[error("S2GetObjectError")]
    GetObjectError(#[from] rusoto_core::RusotoError<GetObjectError>),
    #[error("EmptyObject")]
    EmptyObject,
    #[error("IoError")]
    IoError(#[from] std::io::Error),
    #[error("SerDeError")]
    SerDeError(#[from] SerDeError),
    #[error("DatabaseSerDeError")]
    DatabaseSerDeError(#[from] DatabaseSerDeError),
    #[error("NomadClientError")]
    NomadClientError(#[from] nomad::client::NomadClientError),
    #[error("NomadCliError")]
    NomadCliError(#[from] nomad::cli::NomadCliError),
    #[error("NomadJobAllocationError")]
    NomadJobAllocationError,
    #[error("ArtifactTooLargeError")]
    ArtifactTooLargeError(String),
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
            Error::PutObjectError(_) => Status::internal("Failed to put s3 object"),
            Error::GetObjectError(_) => Status::internal("Failed to get s3 object"),
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
            Error::ArtifactTooLargeError(msg) => Status::invalid_argument(msg),
        }
    }
}
