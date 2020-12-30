use serde::Deserialize;
use sqs_executor::event_decoder::PayloadDecoder;
use std::io::Cursor;

#[derive(Debug, Clone, Default)]
pub struct ZstdJsonDecoder;

impl<D> PayloadDecoder<D> for ZstdJsonDecoder
where
    for<'a> D: Deserialize<'a>,
{
    fn decode(&mut self, body: Vec<u8>) -> Result<D, Box<dyn std::error::Error>> {
        let mut decompressed = Vec::new();

        let mut body = Cursor::new(&body);

        zstd::stream::copy_decode(&mut body, &mut decompressed)?;

        Ok(serde_json::from_slice(&decompressed)?)
    }
}
