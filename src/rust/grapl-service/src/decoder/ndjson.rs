use std::sync::Arc;

use serde::Deserialize;
use sqs_executor::event_decoder::PayloadDecoder;

use crate::decoder::decompress::PayloadDecompressionError;

#[derive(Debug, Clone, Default)]
pub struct NdjsonDecoder;

impl<D> PayloadDecoder<Vec<Result<D, Arc<serde_json::Error>>>> for NdjsonDecoder
where
    for<'a> D: Deserialize<'a>,
{
    type DecoderError = PayloadDecompressionError;

    fn decode(
        &mut self,
        body: Vec<u8>,
    ) -> Result<Vec<Result<D, Arc<serde_json::Error>>>, Self::DecoderError> {
        let decompressed = super::decompress::maybe_decompress(body.as_slice())?;

        let results: Vec<_> = decompressed
            .split(|byte| *byte == b'\n')
            .filter(|line| !line.is_empty())
            .map(|line| serde_json::from_slice(line).map_err(Arc::new))
            .collect();

        Ok(results)
    }
}
