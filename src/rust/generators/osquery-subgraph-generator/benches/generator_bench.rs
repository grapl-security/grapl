use criterion::{
    criterion_group,
    criterion_main,
    Criterion,
};
use grapl_service::decoder::ndjson::NdjsonDecoder;
use osquery_subgraph_generator_lib::{
    generator::OSQuerySubgraphGenerator,
    metrics::OSQuerySubgraphGeneratorMetrics,
};
use sqs_executor::{
    cache::NopCache,
    event_decoder::PayloadDecoder,
    event_handler::{
        CompletedEvents,
        EventHandler,
    },
};
use tokio::runtime::Runtime;

const OSQUERY_SAMPLE_DATA_FILE: &'static str = "sample_data/osquery_data.log";

async fn osquery_generator_process_events(
    osquery_test_events: <OSQuerySubgraphGenerator<NopCache> as EventHandler>::InputEvent,
) {
    let mut generator = OSQuerySubgraphGenerator::new(
        NopCache {},
        OSQuerySubgraphGeneratorMetrics::new("OSQUERY_TEST"),
    );

    let mut completed_events = CompletedEvents { identities: vec![] };

    let _ = generator
        .handle_event(osquery_test_events, &mut completed_events)
        .await;
}

fn bench_osquery_generator_1000_events(c: &mut Criterion) {
    let runtime = Runtime::new().unwrap();

    let osquery_events: Vec<_> = runtime.block_on(async {
        let test_data_bytes = tokio::fs::read(OSQUERY_SAMPLE_DATA_FILE)
            .await
            .expect("Unable to read osquery sample data into test.");

        NdjsonDecoder::default()
            .decode(test_data_bytes)
            .expect("Failed to decode raw data.") // error only occurs on decompression
            .into_iter()
            .take(1_000)
            .collect()
    });

    c.bench_function("OSQuery Generator - 1000 Events", |bencher| {
        bencher.to_async(&runtime).iter(|| async {
            osquery_generator_process_events(osquery_events.clone()).await;
        });
    });
}

criterion_group!(generator_benches, bench_osquery_generator_1000_events);
criterion_main!(generator_benches);
