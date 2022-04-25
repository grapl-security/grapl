use rusoto_s3::{
    GetObjectError,
    PutObjectError,
};
use rust_proto_new::SerDeError;

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
}
