use serde::Deserialize;
use sqs_lambda::event_decoder::PayloadDecoder;
use std::io::Cursor;
use log::*;

// TODO: MOVE THIS INTO A SHARED LIBRARY FOR REUSE BETWEEN GENERIC SUBGRAPH GENERATOR AND THIS GENERATOR
#[derive(Debug, Clone, Default)]
pub struct OSQueryLogDecoder;

impl<D> PayloadDecoder<Vec<D>> for OSQueryLogDecoder
where
    for<'a> D: Deserialize<'a>,
{
    fn decode(&mut self, mut body: Vec<u8>) -> Result<Vec<D>, Box<dyn std::error::Error>> {
        let mut decompressed = Vec::with_capacity(body.len());
        let mut body = Cursor::new(&body);

        zstd::stream::copy_decode(&mut body, &mut decompressed)?;

        let (deserialized_logs, deserialization_errors): (
            Vec<Result<D, _>>,
            Vec<Result<D, _>>,
        ) = decompressed
            .split(|byte| *byte == '\n' as u8)
            .filter(|chunk| !chunk.is_empty())
            .map(|chunk| serde_json::from_slice(chunk))
            .partition(|result| result.is_ok());

        // filter out Nones from these vecs
        let deserialized_logs: Vec<D> = deserialized_logs
            .into_iter()
            .filter_map(|item| item.ok())
            .collect();

        let mut deserialization_errors: Vec<serde_json::Error> = deserialization_errors
            .into_iter()
            .filter_map(|item| item.err())
            .collect();

        for error in &deserialization_errors {
            error!("Deserialization error occurred. {}", error);
        }

        if let Some(error) = deserialization_errors.pop() {
            Err(Box::new(error))
        } else {
            Ok(deserialized_logs)
        }
    }
}
