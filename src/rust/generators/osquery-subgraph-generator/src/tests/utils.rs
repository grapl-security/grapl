use crate::parsers::PartiallyDeserializedOSQueryLog;
use tokio::fs;
use crate::serialization::OSQueryLogDecoder;
use sqs_lambda::event_decoder::PayloadDecoder;

pub(crate) async fn read_osquery_test_data(path: &str) -> Result<Vec<PartiallyDeserializedOSQueryLog>, Box<dyn std::error::Error>> {
    let mut file_data = fs::read(format!("test_data/{}", path)).await
        .expect(&format!("Failed to read test data ({}).", path));
    let mut deserializer = OSQueryLogDecoder::default();

    deserializer.decode(file_data)
}