use std::str::FromStr;

use grapl_service::decoder::decompress::PayloadDecompressionError;
use sqs_executor::{
    errors::{
        CheckedError,
        Recoverable,
    },
    event_decoder::PayloadDecoder,
};
use sysmon::Event;

#[derive(thiserror::Error, Clone, Debug)]
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

impl PayloadDecoder<Vec<Event>> for SysmonDecoder {
    type DecoderError = SysmonDecoderError;

    fn decode(&mut self, body: Vec<u8>) -> Result<Vec<Event>, Self::DecoderError> {
        let decompressed = grapl_service::decoder::decompress::maybe_decompress(body.as_slice())?;

        let mut first_deserialization_error: Option<SysmonDecoderError> = None;

        /*
           This iterator is taking a set of bytes of the logs, splitting the logs on newlines,
           converting the byte sequences to utf-8 strings, and then filtering on supported event
           types: Process Creation, Network Connection, and File Creation.

           https://docs.microsoft.com/en-us/sysinternals/downloads/sysmon#events
        */
        let events: Vec<_> = decompressed
            .split(|byte| *byte == b'\n')
            .map(String::from_utf8_lossy)
            .map(|s| s.to_string())
            .filter_map(|event_str| {
                let parsed_event = Event::from_str(&event_str);
                match parsed_event {
                    Ok(event) => Some(event),
                    Err(error) => {
                        tracing::error!(
                            message = "Unable to deserialize Sysmon event",
                            error =? error,
                            event_str =% event_str
                        );

                        if first_deserialization_error.is_none() {
                            first_deserialization_error =
                                Some(SysmonDecoderError::DeserializeError(error.to_string()))
                        }
                        None
                    }
                }
            })
            .filter(|event| {
                event.is_process_create()
                    || event.is_file_create()
                    || event.is_inbound_network()
                    || event.is_outbound_network()
            })
            .collect();

        // This is a bit awkward at the moment, due to interfaces to the sqs-executor. If some of
        // our events successfully parsed then we want to continue and send those to the event
        // handler. Only if all parsing has failed and we have no events do we want to return an
        // error here.
        match first_deserialization_error {
            Some(error) if events.is_empty() => Err(error),
            _ => Ok(events),
        }
    }
}
