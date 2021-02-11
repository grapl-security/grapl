use std::io::Cursor;

use serde::Deserialize;
use sqs_executor::{
    errors::{CheckedError, Recoverable},
    event_decoder::PayloadDecoder,
};

#[derive(thiserror::Error, Debug)]
pub enum ZstdJsonDecoderError {
    #[error("DecompressionError")]
    DecompressionError(#[from] std::io::Error),
    #[error("ProtoError")]
    JsonError(#[from] serde_json::Error),
}

impl CheckedError for ZstdJsonDecoderError {
    fn error_type(&self) -> Recoverable {
        match self {
            Self::DecompressionError(_) => Recoverable::Persistent,
            Self::JsonError(_) => Recoverable::Persistent,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ZstdJsonDecoder;

impl<D> PayloadDecoder<D> for ZstdJsonDecoder
where
    for<'a> D: Deserialize<'a>,
{
    type DecoderError = ZstdJsonDecoderError;
    fn decode(&mut self, body: Vec<u8>) -> Result<D, Self::DecoderError> {
        let mut decompressed = Vec::new();

        let mut body = Cursor::new(&body);

        zstd::stream::copy_decode(&mut body, &mut decompressed)?;

        Ok(serde_json::from_slice(&decompressed)?)
    }
}
