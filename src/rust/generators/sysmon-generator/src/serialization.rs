use grapl_service::decoder::decompress::PayloadDecompressionError;
use sqs_executor::event_decoder::PayloadDecoder;
use sysmon_parser::SysmonEvent;

#[derive(Debug, Clone, Default)]
pub struct SysmonDecoder;

impl<'a> PayloadDecoder<Vec<SysmonEvent<'static>>> for SysmonDecoder {
    type DecoderError = PayloadDecompressionError;

    #[tracing::instrument(skip(self, body), err)]
    fn decode(&mut self, body: Vec<u8>) -> Result<Vec<SysmonEvent<'static>>, Self::DecoderError> {
        let decompressed = grapl_service::decoder::decompress::maybe_decompress(body.as_slice())?;

        let xml = String::from_utf8_lossy(decompressed.as_slice());
        let sysmon_events: Vec<_> = sysmon_parser::parse_events(xml.as_ref())
            .filter_map(|result| {
                match &result {
                    Ok(sysmon_event) => {
                        tracing::trace!(message = "deserialized Sysmon event", event_type =? sysmon_event.system.event_id);
                    }
                    Err(error) => {
                        tracing::error!(
                            message = "unable to deserialize Sysmon event",
                            %error,
                        );
                    }
                }

                result.ok().map(SysmonEvent::into_owned)
            })
            .collect();

        Ok(sysmon_events)
    }
}
