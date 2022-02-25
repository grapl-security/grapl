use serde::Deserialize;
use sqs_executor::{
    errors::{
        CheckedError,
        Recoverable,
    },
    event_decoder::PayloadDecoder,
};

use crate::decoder::decompress::PayloadDecompressionError;

#[derive(thiserror::Error, Debug)]
pub enum JsonDecoderError {
    #[error("DecompressionError {0}")]
    DecompressionError(#[from] PayloadDecompressionError),
    #[error("JsonError {0}")]
    JsonError(#[from] serde_json::Error),
}

impl CheckedError for JsonDecoderError {
    fn error_type(&self) -> Recoverable {
        match self {
            Self::DecompressionError(_) => Recoverable::Persistent,
            Self::JsonError(_) => Recoverable::Persistent,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct JsonDecoder;

impl<D> PayloadDecoder<D> for JsonDecoder
where
    for<'a> D: Deserialize<'a>,
{
    type DecoderError = JsonDecoderError;
    fn decode(&mut self, body: Vec<u8>) -> Result<D, Self::DecoderError> {
        let decompressed = super::decompress::maybe_decompress(body.as_slice())?;

        serde_json::from_slice(&decompressed).map_err(|e| e.into())
    }
}
