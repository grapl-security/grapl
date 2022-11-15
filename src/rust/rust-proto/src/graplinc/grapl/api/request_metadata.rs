use std::str::FromStr;

use tonic::metadata::{
    AsciiMetadataKey,
    AsciiMetadataValue,
    MetadataMap,
};

use super::client::ClientError;

#[derive(Debug, thiserror::Error)]
pub enum InvalidRequestMetadataError {
    #[error(transparent)]
    InvalidMetadataKey(#[from] tonic::metadata::errors::InvalidMetadataKey),
    #[error(transparent)]
    InvalidMetadataValue(#[from] tonic::metadata::errors::InvalidMetadataValue),
}

type NotYetValidatedKV = (String, String);
type ValidatedKV = (AsciiMetadataKey, AsciiMetadataValue);

// FYI, I'm storing the request metadata here as a vec<tuple> because
// storing it as a map resulted in clippy `error: mutable key type`
#[derive(Clone)]
pub struct RequestMetadata(Vec<ValidatedKV>);

impl RequestMetadata {
    pub fn new(input: Vec<NotYetValidatedKV>) -> Result<Self, ClientError> {
        let validated_input: Result<Vec<ValidatedKV>, InvalidRequestMetadataError> = input
            .into_iter()
            .map(|(key, value)| {
                let key = AsciiMetadataKey::from_str(&key)?;
                let value: AsciiMetadataValue = value.try_into()?;
                Ok((key, value))
            })
            .collect();
        let validated_input = validated_input.map_err(ClientError::from)?;
        Ok(Self(validated_input))
    }

    /// Then, we merge the valid metadata into the tonic::Request's metadata_mut
    pub fn merge_into(self, other: &mut MetadataMap) {
        for (key, value) in self.0.into_iter() {
            other.insert(key, value);
        }
    }
}
