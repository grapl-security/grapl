#![cfg(test)]

use grapl_service::decoder::ZstdJsonDecoder;
use sqs_executor::{
    cache::NopCache, event_decoder::PayloadDecoder, event_handler::CompletedEvents,
};
use tokio::{
    fs::File,
    io::{AsyncReadExt, Result},
};

use crate::{generator::GenericSubgraphGenerator, models::GenericEvent};

#[tokio::test]
/// Tests if generic event serialization is working as expected.
///
/// Verifies that a ProcessStart can be deserialized from a json log and transformed into a Subgraph
async fn test_generic_event_deserialization() {
    let raw_test_string = read_test_data_to_string("events.json")
        .await
        .expect("Failed to read test data for events.json");

    let events: Vec<GenericEvent> = match serde_json::from_str(&raw_test_string) {
        Ok(events) => events,
        Err(e) => panic!(
            "Failed to deserialize event into GenericEvent.\nError: {:?}",
            e
        ),
    };

    // 9 events in events.json
    assert_eq!(events.len(), 9, "Failed to deserialize all log events.");
}

#[tokio::test]
async fn test_log_event_deserialization() {
    let raw_test_data = read_test_data("compressed_events.zstd")
        .await
        .expect("Failed to read test data for compressed_events.zstd");

    let mut generator = GenericSubgraphGenerator::new(NopCache {});

    let mut event_deserializer = ZstdJsonDecoder::default();

    let generic_events: Vec<GenericEvent> = event_deserializer
        .decode(raw_test_data)
        .expect("Failed to deserialize events.");

    let mut completed_events = CompletedEvents::default();

    let result = generator
        .convert_events_to_subgraph(generic_events, &mut completed_events)
        .await;

    match result {
        Err(Err(e)) => {
            panic!("An error occurred during subgraph generation. Err: {:?}", e);
        }
        Err(e) => {
            panic!("An error occurred during subgraph generation. Err: {:?}", e);
        }
        Ok(_) => (),
    }
}

async fn read_test_data_to_string(filename: &str) -> Result<String> {
    let data = read_test_data(filename).await?;

    String::from_utf8(data)
        .map_err(|utf8_error| tokio::io::Error::new(tokio::io::ErrorKind::Other, utf8_error))
}

async fn read_test_data(filename: &str) -> Result<Vec<u8>> {
    let mut file = File::open(format!("test_data/{filename}", filename = filename)).await?;

    let mut contents = vec![];
    file.read_to_end(&mut contents).await?;

    Ok(contents)
}
