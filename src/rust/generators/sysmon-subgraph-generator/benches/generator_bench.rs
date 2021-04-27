#![cfg(test)]
#![feature(test)]

extern crate test;

use std::io::Write;

use grapl_test::cache::EmptyCache;
use sqs_executor::{event_decoder::PayloadDecoder,
                   event_handler::CompletedEvents};
use sysmon_subgraph_generator_lib::{generator::SysmonSubgraphGenerator,
                                    metrics::SysmonSubgraphGeneratorMetrics,
                                    serialization::SysmonDecoder};
use test::Bencher;
use tokio::runtime::Runtime;

const SYSMON_SAMPLE_DATA_FILE: &'static str = "../../../../etc/sample_data/events6.xml";

#[bench]
fn bench_sysmon_generator(bencher: &mut Bencher) {
    let cache = EmptyCache::new();

    let mut generator =
        SysmonSubgraphGenerator::new(cache, SysmonSubgraphGeneratorMetrics::new("SYSMON_TEST"));

    let runtime = Runtime::new().unwrap();

    let sysmon_test_events: Vec<_> = runtime.block_on(async {
        let test_data_bytes = tokio::fs::read(SYSMON_SAMPLE_DATA_FILE)
            .await
            .expect("Unable to read sysmon sample data into test.");

        // Unfortunately, because an error type wraps std::io::Error we cannot clone this data as-is
        // We'll compromise by creating a Vec<Event> and then remapping each iteration.
        // It's not ideal, but shouldn't affect performance too greatly.
        SysmonDecoder::default()
            .decode(test_data_bytes)
            .expect("Unable to parse sysmon sample data into sysmon events.")
            .into_iter()
            .filter_map(|item| item.ok())
            .take(1_000)
            .collect()
    });

    std::io::stdout()
        .write_all(format!("Events: {}\n", sysmon_test_events.len()).as_bytes())
        .expect("Failed to write event count.");

    bencher.iter(|| {
        use sqs_executor::event_handler::EventHandler;

        let mut completed_events = CompletedEvents { identities: vec![] };

        let sysmon_test_data: Vec<_> = sysmon_test_events
            .clone()
            .into_iter()
            .map(|event| Ok(event))
            .collect();

        let _ = runtime.block_on(generator.handle_event(sysmon_test_data, &mut completed_events));
    });
}
