use rusoto_s3::{
    GetObjectError,
    PutObjectError,
};
use rust_proto::plugin_registry::PluginRegistryDeserializationError;

use crate::nomad;

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
    #[error("PluginRegistryDeserializationError")]
    PluginRegistryDeserializationError(#[from] PluginRegistryDeserializationError),
    #[error("NomadClientError")]
    NomadClientError(#[from] nomad::client::NomadClientError),
    #[error("NomadCliError")]
    NomadCliError(#[from] nomad::cli::NomadCliError),
    #[error("NomadJobAllocationError")]
    NomadJobAllocationError,
}
