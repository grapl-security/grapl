use sqs_lambda::event_decoder::PayloadDecoder;
use std::io::Cursor;

/// A [PayloadDecoder] used to decompress zstd encoded events sent to an [EventHandler].
///
/// This `struct` is typically used in conjunction with a subsequent call to [run_graph_generator].
#[derive(Debug, Clone, Default)]
pub struct ZstdDecoder;

impl PayloadDecoder<Vec<u8>> for ZstdDecoder {
    fn decode(&mut self, body: Vec<u8>) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut decompressed = Vec::new();
        let mut body = Cursor::new(&body);

        zstd::stream::copy_decode(&mut body, &mut decompressed)?;

        Ok(decompressed)
    }
}
