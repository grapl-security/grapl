use grapl_service::decoder::NdjsonDecoder;
use sqs_executor::event_decoder::PayloadDecoder;
use tokio::fs;

use crate::parsers::PartiallyDeserializedOSQueryLog;

#[cfg(test)]
pub(crate) async fn read_osquery_test_data(
    path: &str,
) -> Vec<Result<PartiallyDeserializedOSQueryLog, serde_json::Error>> {
    let file_data = fs::read(format!("test_data/{}", path))
        .await
        .expect(&format!("Failed to read test data ({}).", path));
    let mut decoder = NdjsonDecoder::default();

    decoder.decode(file_data).unwrap()
}
