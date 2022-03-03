use grapl_service::decoder::decompress::PayloadDecompressionError;
use sqs_executor::{
    errors::{
        CheckedError,
        Recoverable,
    },
    event_decoder::PayloadDecoder,
};
use sysmon_parser::{
    system::EventId,
    SysmonEvent,
};

#[derive(thiserror::Error, Clone, Debug)]
pub enum SysmonDecoderError {
    #[error("DeserializeError {0}")]
    DeserializeError(String),
    #[error("DecompressionError {0}")]
    DecompressionError(#[from] PayloadDecompressionError),
    #[error("TimeError {0}")]
    TimeError(#[from] chrono::ParseError),
    #[error("Utf8Error {0}")]
    Utf8Error(#[from] std::str::Utf8Error),
}

impl CheckedError for SysmonDecoderError {
    fn error_type(&self) -> Recoverable {
        match self {
            Self::DeserializeError(_) => Recoverable::Persistent,
            Self::DecompressionError(_) => Recoverable::Persistent,
            Self::TimeError(_) => Recoverable::Persistent,
            Self::Utf8Error(_) => Recoverable::Persistent,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct SysmonDecoder;

impl<'a> PayloadDecoder<Vec<SysmonEvent<'static>>> for SysmonDecoder {
    type DecoderError = SysmonDecoderError;

    #[tracing::instrument(skip(self), err)]
    fn decode(&mut self, body: Vec<u8>) -> Result<Vec<SysmonEvent<'static>>, Self::DecoderError> {
        let decompressed = grapl_service::decoder::decompress::maybe_decompress(body.as_slice())?;

        let mut first_deserialization_error: Option<SysmonDecoderError> = None;

        let xml = String::from_utf8_lossy(decompressed.as_slice());
        let sysmon_events: Vec<_> = sysmon_parser::parse_events(xml.as_ref())
            .filter_map(|result| {
                match &result {
                    Ok(_) => {
                        tracing::debug!(message = "Deserialized sysmon event");
                    }
                    Err(error) => {
                        tracing::error!(
                            message = "Unable to deserialize Sysmon event",
                            error =? error,
                        );

                        if first_deserialization_error.is_none() {
                            first_deserialization_error =
                                Some(SysmonDecoderError::DeserializeError(error.to_string()))
                        }
                    }
                }

                result.ok()
            })
            .filter(|event| {
                matches!(
                    event.system.event_id,
                    EventId::ProcessCreation
                        | EventId::ProcessTerminated
                        | EventId::FileCreate
                        | EventId::NetworkConnection
                )
            })
            .map(|event| event.into_owned())
            .collect();

        // This is a bit awkward at the moment, due to interfaces to the sqs-executor. If some of
        // our events successfully parsed then we want to continue and send those to the event
        // handler. Only if all parsing has failed and we have no events do we want to return an
        // error here.
        match first_deserialization_error {
            Some(error) if sysmon_events.is_empty() => Err(error),
            _ => Ok(sysmon_events),
        }
    }
}
