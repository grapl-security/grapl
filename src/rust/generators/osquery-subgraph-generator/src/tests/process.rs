use crate::generator::OSQuerySubgraphGenerator;
use crate::metrics::OSQuerySubgraphGeneratorMetrics;
use crate::tests::utils;
use regex::internal::Input;
use sqs_executor::cache::NopCache;
use sqs_executor::event_handler::EventHandler;
use grapl_service::decoder::ZstdJsonDecoder;

#[tokio::test]
async fn test_subgraph_generation_process_create() {
    let metrics = OSQuerySubgraphGeneratorMetrics::new("osquery-subgraph-generator");
    let mut generator = OSQuerySubgraphGenerator::new(NopCache {}, metrics);

    let logs = utils::read_osquery_test_data("process_create.zstd")
        .await
        .expect("Failed to parse process_create.zstd logs into OSQueryLogs.");

    let output_event = generator.handle_event(logs).await;

    match &output_event.completed_event {
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
