use std::io::Cursor;

use sqs_executor::{errors::{CheckedError,
                            Recoverable},
                   event_decoder::PayloadDecoder};

#[derive(thiserror::Error, Debug)]
pub enum ZstdDecoderError {
    #[error("DecompressionError")]
    DecompressionError(#[from] std::io::Error),
}

impl CheckedError for ZstdDecoderError {
    fn error_type(&self) -> Recoverable {
        match self {
            Self::DecompressionError(_) => Recoverable::Persistent,
        }
    }
}

/// A [PayloadDecoder] used to decompress zstd encoded events sent to an [EventHandler].
///
/// This `struct` is typically used in conjunction with a subsequent call to [run_graph_generator].
#[derive(Debug, Clone, Default)]
pub struct ZstdDecoder;

impl PayloadDecoder<Vec<u8>> for ZstdDecoder {
    type DecoderError = ZstdDecoderError;

    fn decode(&mut self, body: Vec<u8>) -> Result<Vec<u8>, Self::DecoderError> {
        let mut decompressed = Vec::with_capacity(body.len());
        let mut body = Cursor::new(&body);

        zstd::stream::copy_decode(&mut body, &mut decompressed)?;

        Ok(decompressed)
    }
}
