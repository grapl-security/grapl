use std::str::FromStr;

use tonic::metadata::{
    AsciiMetadataKey,
    AsciiMetadataValue,
    MetadataMap,
};

#[derive(Debug, thiserror::Error)]
pub enum InvalidRequestMetadataError {
    #[error(transparent)]
    InvalidMetadataKey(#[from] tonic::metadata::errors::InvalidMetadataKey),
    #[error(transparent)]
    InvalidMetadataValue(#[from] tonic::metadata::errors::InvalidMetadataValue),
}

type UnvalidatedKV = (String, String);
type ValidatedKV = (AsciiMetadataKey, AsciiMetadataValue);

pub struct RequestMetadata(Vec<UnvalidatedKV>);

impl RequestMetadata {
    pub fn new(input: Vec<UnvalidatedKV>) -> Self {
        Self(input)
    }

    /// First, validate the user input at `execute()` time, and return an
    /// Err if it's invalid metadata.
    pub fn validate(self) -> Result<ValidatedRequestMetadata, InvalidRequestMetadataError> {
        let mapping: Result<Vec<ValidatedKV>, InvalidRequestMetadataError> = self
            .0
            .into_iter()
            .map(|(key, value)| {
                let key = AsciiMetadataKey::from_str(&key)?;
                let value: AsciiMetadataValue = value.try_into()?;
                Ok((key, value))
            })
            .collect();
        Ok(ValidatedRequestMetadata(mapping?))
    }
}

// FYI, I'm storing the request metadata here as a vec<tuple> because
// storing it as a map resulted in clippy `error: mutable key type`
#[derive(Clone)]
pub struct ValidatedRequestMetadata(Vec<ValidatedKV>);
impl ValidatedRequestMetadata {
    /// Then, we merge the valid metadata into the tonic::Request's metadata_mut
    pub fn merge_into(self, other: &mut MetadataMap) {
        for (key, value) in self.0.into_iter() {
            other.insert(key, value);
        }
    }
}
