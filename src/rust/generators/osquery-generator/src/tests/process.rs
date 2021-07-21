use sqs_executor::{
    cache::NopCache,
    event_handler::{
        CompletedEvents,
        EventHandler,
    },
};

use crate::{
    generator::OSQueryGenerator,
    metrics::OSQueryGeneratorMetrics,
    tests::utils,
};

#[tokio::test]
async fn test_subgraph_generation_process_create() {
    let metrics = OSQueryGeneratorMetrics::new("osquery-generator");
    let mut generator = OSQueryGenerator::new(NopCache {}, metrics);

    let logs = utils::read_osquery_test_data("process_create.zstd").await;

    let mut completion = CompletedEvents::default();
    let output_event = generator.handle_event(logs, &mut completion).await;

    match &output_event {
        Ok(subgraph) => {
            assert!(!subgraph.is_empty(), "Generated subgraph was empty.")
        }
        Err(Ok((subgraph, e))) => {
            assert!(
                !subgraph.is_empty(),
                "Generated subgraph was empty and errors were generated"
            );
            panic!(
                "OSQuery subgraph generator failed to generate subgraph with error: {}",
                e
            );
        }
        Err(Err(e)) => panic!(
            "OSQuery subgraph generator failed to generate subgraph with error: {}",
            e
        ),
    };
}
