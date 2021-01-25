use sqs_lambda::{cache::NopCache,
                 event_handler::{Completion,
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

    let output_event = generator.handle_event(logs).await;

    match &output_event.completed_event {
        Completion::Total(subgraph) => {
            assert!(!subgraph.is_empty(), "Generated subgraph was empty.")
        }
        Completion::Partial((subgraph, e)) => {
            assert!(
                !subgraph.is_empty(),
                "Generated subgraph was empty and errors were generated"
            );
            panic!(
                "OSQuery subgraph generator failed to generate subgraph with error: {}",
                e
            );
        }
        Completion::Error(e) => panic!(
            "OSQuery subgraph generator failed to generate subgraph with error: {}",
            e
        ),
    };
}
