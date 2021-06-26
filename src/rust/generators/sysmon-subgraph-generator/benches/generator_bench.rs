use criterion::{
    criterion_group,
    criterion_main,
    Criterion,
};
use sqs_executor::{
    cache::NopCache,
    event_decoder::PayloadDecoder,
    event_handler::{
        CompletedEvents,
        EventHandler,
    },
};
use sysmon_subgraph_generator_lib::{
    generator::SysmonSubgraphGenerator,
    metrics::SysmonSubgraphGeneratorMetrics,
    serialization::SysmonDecoder,
};
use tokio::runtime::Runtime;

const SYSMON_SAMPLE_DATA_FILE: &'static str = "sample_data/events6.xml";

async fn sysmon_generator_process_events(
    sysmon_test_events: <SysmonSubgraphGenerator<NopCache> as EventHandler>::InputEvent,
) {
    let mut generator = SysmonSubgraphGenerator::new(
        NopCache {},
        SysmonSubgraphGeneratorMetrics::new("SYSMON_TEST"),
    );

    let mut completed_events = CompletedEvents { identities: vec![] };

    let _ = generator
        .handle_event(sysmon_test_events, &mut completed_events)
        .await;
}

fn bench_sysmon_generator_1000_events(c: &mut Criterion) {
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
            .take(1_000)
            .collect()
    });

    c.bench_function("Sysmon Generator - 1000 Events", |bencher| {
        bencher.to_async(&runtime).iter(|| async {
            sysmon_generator_process_events(sysmon_test_events.clone()).await;
        });
    });
}

criterion_group!(generator_benches, bench_sysmon_generator_1000_events);
criterion_main!(generator_benches);
