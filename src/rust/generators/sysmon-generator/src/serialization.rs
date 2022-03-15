use sqs_executor::{
    event_decoder::PayloadDecoder,
};
use sysmon_parser::{
    system::EventId,
    SysmonEvent,
};

use grapl_service::decoder::decompress::PayloadDecompressionError;

#[derive(Debug, Clone, Default)]
pub struct SysmonDecoder;

impl<'a> PayloadDecoder<Vec<SysmonEvent<'static>>> for SysmonDecoder {
    type DecoderError = PayloadDecompressionError;

    #[tracing::instrument(skip(self), err)]
    fn decode(&mut self, body: Vec<u8>) -> Result<Vec<SysmonEvent<'static>>, Self::DecoderError> {
        let decompressed = grapl_service::decoder::decompress::maybe_decompress(body.as_slice())?;

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
                            %error,
                        );
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

        Ok(sysmon_events)
    }
}
