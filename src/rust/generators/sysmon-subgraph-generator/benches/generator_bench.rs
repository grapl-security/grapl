#![cfg(test)]
#![feature(test)]

extern crate test;

use test::Bencher;
use sysmon_subgraph_generator_lib::generator::SysmonSubgraphGenerator;
use sysmon_subgraph_generator_lib::metrics::SysmonSubgraphGeneratorMetrics;
use sqs_executor::cache::{
    Cache,
    Cacheable
};
use sqs_executor::errors::{
    CheckedError,
    Recoverable
};
use sqs_executor::event_handler::CompletedEvents;
use async_trait::async_trait;
use std::error::Error;
use std::fmt::{Display, Formatter, Result as FmtResult};
use tokio::runtime::Runtime;
use grapl_test::cache::EmptyCache;

const SYSMON_SAMPLE_DATA_FILE: &'static str = "../../../../etc/sample_data/events6.xml";

#[bench]
fn bench_sysmon_generator(bencher: &mut Bencher) {
    let cache = EmptyCache::new();

    let mut generator = SysmonSubgraphGenerator::new(
        cache,
        SysmonSubgraphGeneratorMetrics::new("SYSMON_TEST")
    );

    let runtime = Runtime::new().unwrap();

    let test_data_source = runtime.block_on( async {
        tokio::fs::read(SYSMON_SAMPLE_DATA_FILE).await
            .expect("Unable to read sysmon sample data into test.")
    });

    bencher.iter(|| {
        use sqs_executor::event_handler::EventHandler;

        let mut completed_events = CompletedEvents { identities: vec![] };
        let _ = runtime.block_on(generator.handle_event(test_data_source.clone(), &mut completed_events));
    });
}
