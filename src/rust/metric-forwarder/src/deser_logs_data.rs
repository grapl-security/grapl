use aws_lambda_events::event::cloudwatch_logs::{
    CloudwatchLogsData,
    CloudwatchLogsEvent,
};

use crate::error::{
    MetricForwarderError,
    MetricForwarderError::{
        DecodeBase64Error,
        GunzipToStringError,
        ParseStringToLogsdataError,
        PoorlyFormattedEventError,
    },
};

fn parse_string_to_logsdata(gunzipped: String) -> Result<CloudwatchLogsData, MetricForwarderError> {
    use serde_json::from_str;

    from_str(&gunzipped).map_err(|err| ParseStringToLogsdataError(err.to_string()))
}

fn base64_decode_raw_log_to_gzip(data: &str) -> Result<Vec<u8>, MetricForwarderError> {
    use base64::decode;

    decode(data).map_err(|err| DecodeBase64Error(err.to_string()))
}

fn gunzip_to_string(gzipped: Vec<u8>) -> Result<String, MetricForwarderError> {
    use std::io::Read;

    use flate2::read::GzDecoder;

    let mut raw_data = String::new();
    match GzDecoder::new(gzipped.as_slice()).read_to_string(&mut raw_data) {
        Ok(_) => Ok(raw_data),
        Err(err) => Err(GunzipToStringError(err.to_string())),
    }
}

pub fn aws_event_to_cloudwatch_logs_data(
    event: CloudwatchLogsEvent,
) -> Result<CloudwatchLogsData, MetricForwarderError> {
    if let Some(data) = event.aws_logs.data {
        let result = base64_decode_raw_log_to_gzip(&data)
            .and_then(gunzip_to_string)
            .and_then(parse_string_to_logsdata);
        result
    } else {
        Err(PoorlyFormattedEventError())
    }
}
