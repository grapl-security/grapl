use sqs_executor::{cache::NopCache,
                   event_handler::{CompletedEvents,
                                   EventHandler}};

use crate::{generator::OSQuerySubgraphGenerator,
            metrics::OSQuerySubgraphGeneratorMetrics,
            tests::utils};

#[tokio::test]
async fn test_subgraph_generation_process_create() {
    let metrics = OSQuerySubgraphGeneratorMetrics::new("osquery-subgraph-generator");
    let mut generator = OSQuerySubgraphGenerator::new(NopCache {}, metrics);

    let logs = utils::read_osquery_test_data("process_create.zstd")
        .await
        .expect("Failed to parse process_create.zstd logs into OSQueryLogs.");

    let mut completion = CompletedEvents::default();
    let output_event = generator.handle_event(logs, &mut completion).await;

    match &output_event {
        Ok(subgraph) => {
            assert!(!subgraph.is_empty(), "Generated subgraphwas empty.")
        }
        Err(Ok((subgraph, e))) => {
            assert!(
                !subgraph.is_empty(),
                "Generated subgraphwas empty and errors were generated"
            );
            panic!(
                "OSQuery subgraphgenerator failed to generate subgraphwith error: {}",
                e
            );
        }
        Err(Err(e)) => panic!(
            "OSQuery subgraphgenerator failed to generate subgraphwith error: {}",
            e
        ),
    };
}
