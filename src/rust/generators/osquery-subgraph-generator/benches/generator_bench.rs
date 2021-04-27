#![cfg(test)]
#![feature(test)]

extern crate test;

use test::Bencher;
use grapl_test::cache::EmptyCache;
use osquery_subgraph_generator_lib::generator::OSQuerySubgraphGenerator;
use osquery_subgraph_generator_lib::metrics::OSQuerySubgraphGeneratorMetrics;
use osquery_subgraph_generator_lib::parsers::PartiallyDeserializedOSQueryLog;
use tokio::runtime::Runtime;
use sqs_executor::event_handler::CompletedEvents;

const OSQUERY_SAMPLE_DATA_FILE: &'static str = "../../../../etc/sample_data/osquery_data.log";

#[bench]
fn bench_osquery_generator(bencher: &mut Bencher) {
    let cache = EmptyCache::new();

    let mut generator =  OSQuerySubgraphGenerator::new(
        cache,
        OSQuerySubgraphGeneratorMetrics::new("SYSMON_TEST")
    );

    let runtime = Runtime::new().unwrap();

    let test_data_bytes = runtime.block_on( async {
        tokio::fs::read(OSQUERY_SAMPLE_DATA_FILE).await
            .expect("Unable to read osquery sample data into test.")
    });

    let test_data: Vec<_> = test_data_bytes
        .split(|byte| *byte == b'\n')
        .filter(|chunk| !chunk.is_empty())
        .filter_map(|chunk| serde_json::from_slice::<PartiallyDeserializedOSQueryLog>(chunk).ok())
        .collect();

    bencher.iter(|| {
        use sqs_executor::event_handler::EventHandler;

        let mut completed_events = CompletedEvents { identities: vec![] };
        let _ = runtime.block_on(generator.handle_event(test_data.clone(), &mut completed_events));
    });
}