use grapl_service::decoder::decompress::PayloadDecompressionError;
use sqs_executor::{errors::{CheckedError,
                            Recoverable},
                   event_decoder::PayloadDecoder};
use sysmon::Event;

#[derive(thiserror::Error, Debug)]
pub enum SysmonDecoderError {
    #[error("DeserializeError")]
    DeserializeError(String),
    #[error("DecompressionError")]
    DecompressionError(#[from] PayloadDecompressionError),
    #[error("TimeError")]
    TimeError(#[from] chrono::ParseError),
}

impl CheckedError for SysmonDecoderError {
    fn error_type(&self) -> Recoverable {
        match self {
            Self::DeserializeError(_) => Recoverable::Persistent,
            Self::DecompressionError(_) => Recoverable::Persistent,
            Self::TimeError(_) => Recoverable::Persistent,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct SysmonDecoder;

impl PayloadDecoder<Vec<Result<Event, SysmonDecoderError>>> for SysmonDecoder {
    type DecoderError = SysmonDecoderError;

    fn decode(
        &mut self,
        body: Vec<u8>,
    ) -> Result<Vec<Result<Event, Self::DecoderError>>, Self::DecoderError> {
        let decompressed = grapl_service::decoder::decompress::maybe_decompress(body.as_slice())?;

        /*
           This iterator is taking a set of bytes of the logs, splitting the logs on newlines,
           converting the byte sequences to utf-8 strings, and then filtering on the following criteria:
               1. The line isn't empty
               2. The line is not `\n` (to prevent issues with multiple newline sequences)
               3. The line contains event with ID 1, 3, or 11

           The event ids 1, 3, and 11 correspond to Process Creation, Network Connection, and File Creation
           in that order.

           https://docs.microsoft.com/en-us/sysinternals/downloads/sysmon#events
        */
        let events: Vec<Result<Event, Self::DecoderError>> = decompressed
            .split(|byte| *byte == b'\n')
            .map(String::from_utf8_lossy)
            .map(|s| s.to_string())
            .filter(|event| {
                (!event.is_empty() && event != "\n")
                    && (event.contains(&"EventID>1<"[..])
                        || event.contains(&"EventID>3<"[..])
                        || event.contains(&"EventID>11<"[..]))
            })
            .map(|event| {
                Event::from_str(event)
                    .map_err(|e| SysmonDecoderError::DeserializeError(e.to_string()))
            })
            .collect();

        Ok(events)
    }
}
