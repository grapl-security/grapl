use prost::{
    DecodeError,
    Message,
};
use sqs_executor::{
    errors::{
        CheckedError,
        Recoverable,
    },
    event_decoder::PayloadDecoder,
};

use crate::decoder::decompress::PayloadDecompressionError;

#[derive(thiserror::Error, Debug)]
pub enum ProtoDecoderError {
    #[error("DecompressionError {0}")]
    DecompressionError(#[from] PayloadDecompressionError),
    #[error("ProtoError {0}")]
    ProtoError(#[from] DecodeError),
}

impl CheckedError for ProtoDecoderError {
    fn error_type(&self) -> Recoverable {
        match self {
            Self::DecompressionError(_) => Recoverable::Persistent,
            Self::ProtoError(_) => Recoverable::Persistent,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ProtoDecoder;

impl<E> PayloadDecoder<E> for ProtoDecoder
where
    E: Message + Default,
{
    type DecoderError = ProtoDecoderError;
    fn decode(&mut self, body: Vec<u8>) -> Result<E, Self::DecoderError>
    where
        E: Message + Default,
    {
        let decompressed = super::decompress::maybe_decompress(body.as_slice())?;

        let buf = prost::bytes::Bytes::from(decompressed);
        E::decode(buf).map_err(|e| e.into())
    }
}
