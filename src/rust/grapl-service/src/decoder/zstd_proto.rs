use std::io::Cursor;

use prost::{DecodeError, Message};
use sqs_executor::{
    errors::{CheckedError, Recoverable},
    event_decoder::PayloadDecoder,
};

#[derive(thiserror::Error, Debug)]
pub enum ZstdProtoDecoderError {
    #[error("DecompressionError")]
    DecompressionError(#[from] std::io::Error),
    #[error("ProtoError")]
    ProtoError(#[from] DecodeError),
}

impl CheckedError for ZstdProtoDecoderError {
    fn error_type(&self) -> Recoverable {
        match self {
            Self::DecompressionError(_) => Recoverable::Persistent,
            Self::ProtoError(_) => Recoverable::Persistent,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ZstdProtoDecoder;

impl<E> PayloadDecoder<E> for ZstdProtoDecoder
where
    E: Message + Default,
{
    type DecoderError = ZstdProtoDecoderError;
    fn decode(&mut self, body: Vec<u8>) -> Result<E, Self::DecoderError>
    where
        E: Message + Default,
    {
        let mut decompressed = Vec::new();

        let mut body = Cursor::new(&body);

        zstd::stream::copy_decode(&mut body, &mut decompressed)?;

        let buf = prost::bytes::Bytes::from(decompressed);
        Ok(E::decode(buf)?)
    }
}
