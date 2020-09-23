use tokio::fs::File;
use tokio::io::{Result, AsyncReadExt};
use crate::models::GenericEvent;
use graph_descriptions::graph_description::*;
use crate::generator::GenericSubgraphGenerator;
use sqs_lambda::cache::NopCache;
use crate::serialization::ZstdJsonDecoder;
use sqs_lambda::event_decoder::PayloadDecoder;

#[tokio::test]
/// Tests if generic event serialization is working as expected.
///
/// Verifies that a ProcessStart can be deserialized from a json log and transformed into a Subgraph
async fn test_generic_event_deserialization() {
    let raw_test_string = read_test_data_to_string("process_start.json").await
        .expect("Failed to read test data for process_start.json");

    let event: GenericEvent = match serde_json::from_str(&raw_test_string) {
        Ok(event) => event,
        Err(e) => panic!("Failed to deserialize event into GenericEvent.\nError: {}", e)
    };

    let process_start = match event {
        GenericEvent::ProcessStart(process_start) => process_start,
        _ => panic!("Failed to deserialize event into correct enum variant.")
    };

    // verify that this event has enough information to properly transform into a subgraph
    let graph: Graph = Graph::from(process_start);

    assert!(graph.nodes.len() > 0);
    assert!(graph.edges.len() > 0);
}

#[tokio::test]
async fn test_log_event_deserialization() {
    let raw_test_data = read_test_data("compressed_process_start.zstd").await
        .expect("Failed to read test data for process_start.json");

    let mut event_deserializer = ZstdJsonDecoder::default();

    let generic_event: GenericEvent = event_deserializer.decode(raw_test_data)
        .expect("Failed to deserialize process start event.");

    match generic_event {
        GenericEvent::ProcessStart(_) => { },
        _ => panic!("Deserialized event into wrong variant.")
    };
}


async fn read_test_data_to_string(filename: &str) -> Result<String> {
    let data = read_test_data(filename).await?;

    String::from_utf8(data).map_err(|utf8_error| {
        tokio::io::Error::new(tokio::io::ErrorKind::Other, utf8_error)
    })
}


async fn read_test_data(filename: &str) -> Result<Vec<u8>> {
    let mut file = File::open(format!("test_data/{filename}", filename=filename)).await?;

    let mut contents = vec![];
    file.read_to_end(&mut contents).await?;

    Ok(contents)
}