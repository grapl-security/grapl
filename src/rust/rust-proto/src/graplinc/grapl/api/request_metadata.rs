use std::str::FromStr;

use tonic::metadata::MetadataMap;

#[derive(Debug, thiserror::Error)]
pub enum InvalidRequestMetadataError {
    #[error(transparent)]
    InvalidMetadataKey(#[from] tonic::metadata::errors::InvalidMetadataKey),
    #[error(transparent)]
    InvalidMetadataValue(#[from] tonic::metadata::errors::InvalidMetadataValue),
}

pub struct RequestMetadata(std::collections::HashMap<String, String>);
impl RequestMetadata {
    pub fn validate(self) -> Result<ValidatedRequestMetadata, InvalidRequestMetadataError>{
        let validated_metadata = tonic::metadata::MetadataMap::new();

        for (key, value) in self.0.into_iter() {
            let key = tonic::metadata::AsciiMetadataKey::from_str(&key)?;
            let value: tonic::metadata::AsciiMetadataValue = value.try_into()?;
            validated_metadata.insert(key, value);
        }
        Ok(ValidatedRequestMetadata(validated_metadata))
    }
}

#[derive(Clone)]
pub struct ValidatedRequestMetadata(MetadataMap);
impl ValidatedRequestMetadata {
    pub fn merge_into(self, other: &mut MetadataMap) {
        for (key, value) in self.0.into_iter() {
            other.insert(key, value);
        }
    }
}