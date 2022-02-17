use serde::Deserialize;
use sqs_executor::{
    errors::{CheckedError, Recoverable},
    event_decoder::PayloadDecoder,
};

use crate::decoder::decompress::PayloadDecompressionError;

#[derive(Debug, Clone, Default)]
pub struct NdjsonDecoder;

#[derive(thiserror::Error, Debug)]
pub enum NdjsonDecoderError {
    #[error("DecompressionError {0}")]
    Decompression(#[from] PayloadDecompressionError),
    #[error("DeserializeError {0}")]
    Deserialization(#[from] serde_json::Error),
}

impl CheckedError for NdjsonDecoderError {
    fn error_type(&self) -> Recoverable {
        match self {
            Self::Decompression(_) => Recoverable::Persistent,
            Self::Deserialization(_) => Recoverable::Persistent,
        }
    }
}

impl<D> PayloadDecoder<Vec<D>> for NdjsonDecoder
where
    for<'a> D: Deserialize<'a>,
{
    type DecoderError = NdjsonDecoderError;

    fn decode(&mut self, body: Vec<u8>) -> Result<Vec<D>, Self::DecoderError> {
        let decompressed = super::decompress::maybe_decompress(body.as_slice())?;

        let mut first_deserialization_error: Option<serde_json::Error> = None;

        let events: Vec<_> = decompressed
            .split(|byte| *byte == b'\n')
            .filter(|line| !line.is_empty())
            .filter_map(|line| {
                let result = serde_json::from_slice(line);
                match result {
                    Ok(result) => Some(result),
                    Err(error) => {
                        tracing::error!(message="Unable to deserialize OSQuery event.", error=?error);

                        if first_deserialization_error.is_none() {
                            first_deserialization_error = Some(error);
                        }
                        None
                    }
                }
            })
            .collect();

        // This is a bit awkward at the moment, due to interfaces to the sqs-executor.  If some of
        // our events successfully parsed then we want to continue and send those to the event
        // handler. Only if all parsing has failed and we have no events do we want to return an
        // error here. Note the other error condition is a failure to decompress above.
        match first_deserialization_error {
            Some(error) if events.is_empty() => Err(error.into()),
            _ => Ok(events),
        }
    }
}
