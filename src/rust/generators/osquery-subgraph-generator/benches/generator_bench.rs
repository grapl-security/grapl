#![cfg(test)]
#![feature(test)]

extern crate test;

use std::io::Write;

use grapl_test::cache::EmptyCache;
use osquery_subgraph_generator_lib::{generator::OSQuerySubgraphGenerator,
                                     metrics::OSQuerySubgraphGeneratorMetrics,
                                     parsers::PartiallyDeserializedOSQueryLog};
use sqs_executor::event_handler::CompletedEvents;
use test::Bencher;
use tokio::runtime::Runtime;

const OSQUERY_SAMPLE_DATA_FILE: &'static str = "../../../../etc/sample_data/osquery_data.log";

#[bench]
fn bench_osquery_generator(bencher: &mut Bencher) {
    let cache = EmptyCache::new();

    let mut generator =
        OSQuerySubgraphGenerator::new(cache, OSQuerySubgraphGeneratorMetrics::new("SYSMON_TEST"));

    let runtime = Runtime::new().unwrap();

    let test_data_bytes = runtime.block_on(async {
        tokio::fs::read(OSQUERY_SAMPLE_DATA_FILE)
            .await
            .expect("Unable to read osquery sample data into test.")
    });

    let osquery_events: Vec<_> = test_data_bytes
        .split(|byte| *byte == b'\n')
        .filter(|chunk| !chunk.is_empty())
        .filter_map(|chunk| serde_json::from_slice::<PartiallyDeserializedOSQueryLog>(chunk).ok())
        .take(1_000)
        .collect();

    std::io::stdout()
        .write_all(format!("Events: {}\n", osquery_events.len()).as_bytes())
        .expect("Failed to write number of events");

    bencher.iter(|| {
        use sqs_executor::event_handler::EventHandler;

        let mut completed_events = CompletedEvents { identities: vec![] };

        let osquery_test_events: Vec<_> = osquery_events
            .clone()
            .into_iter()
            .map(|event| Ok(event))
            .collect();

        let _ =
            runtime.block_on(generator.handle_event(osquery_test_events, &mut completed_events));
    });
}
