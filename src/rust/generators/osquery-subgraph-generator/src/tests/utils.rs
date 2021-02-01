use sqs_lambda::event_decoder::PayloadDecoder;
use tokio::fs;

use crate::{parsers::PartiallyDeserializedOSQueryLog,
            serialization::OSQueryLogDecoder};

pub(crate) async fn read_osquery_test_data(
    path: &str,
) -> Result<Vec<PartiallyDeserializedOSQueryLog>, Box<dyn std::error::Error>> {
    let file_data = fs::read(format!("test_data/{}", path))
        .await
        .expect(&format!("Failed to read test data ({}).", path));
    let mut deserializer = OSQueryLogDecoder::default();

    deserializer.decode(file_data)
}
