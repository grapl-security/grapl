use std::sync::Arc;

use grapl_service::decoder::NdjsonDecoder;
use sqs_executor::event_decoder::PayloadDecoder;
use tokio::fs;

use crate::parsers::PartiallyDeserializedOSQueryLog;

#[cfg(test)]
pub(crate) async fn read_osquery_test_data(
    path: &str,
) -> Vec<Result<PartiallyDeserializedOSQueryLog, Arc<serde_json::Error>>> {
    let file_data = fs::read(format!("sample_data/{}", path))
        .await
        .unwrap_or_else(|_| panic!("Failed to read test data ({}).", path));
    let mut decoder = NdjsonDecoder::default();

    decoder.decode(file_data).unwrap()
}
