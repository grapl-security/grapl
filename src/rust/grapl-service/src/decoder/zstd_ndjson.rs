use std::io::Cursor;

use serde::Deserialize;
use sqs_executor::{errors::{CheckedError,
                            Recoverable},
                   event_decoder::PayloadDecoder};

#[derive(thiserror::Error, Debug)]
pub enum ZstdNdjsonDecoderError {
    #[error("DecompressionError")]
    DecompressionError(#[from] std::io::Error),
}

impl CheckedError for ZstdNdjsonDecoderError {
    fn error_type(&self) -> Recoverable {
        match self {
            Self::DecompressionError(_) => Recoverable::Persistent,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ZstdNdjsonDecoder;

impl<D> PayloadDecoder<Vec<Result<D, serde_json::Error>>> for ZstdNdjsonDecoder
where
    for<'a> D: Deserialize<'a>,
{
    type DecoderError = ZstdNdjsonDecoderError;
    fn decode(
        &mut self,
        body: Vec<u8>,
    ) -> Result<Vec<Result<D, serde_json::Error>>, Self::DecoderError> {
        let mut decompressed = Vec::new();

        let mut body = Cursor::new(&body);

        zstd::stream::copy_decode(&mut body, &mut decompressed)?;

        let results: Vec<_> = decompressed
            .split(|byte| *byte == b'\n')
            .filter(|line| !line.is_empty())
            .map(|line| serde_json::from_slice(line))
            .collect();

        Ok(results)
    }
}
