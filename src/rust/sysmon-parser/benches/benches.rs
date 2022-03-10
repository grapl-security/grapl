use criterion::{
    black_box,
    criterion_group,
    criterion_main,
    Criterion,
};
use sysmon_parser::SysmonEvent;

pub fn bulk_bench(c: &mut Criterion) {
    let events6 = std::fs::read_to_string("tests/data/events6.xml").unwrap();

    c.bench_function("bulk - events6", |b| {
        b.iter(|| {
            let results = sysmon_parser::parse_events(&events6);
            for result in results {
                black_box(result.unwrap());
            }
        })
    });
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let xml = std::fs::read_to_string("tests/data/process_creation.xml").unwrap();

    c.bench_function("event_data - process creation - no unescape", |b| {
        b.iter(|| SysmonEvent::from_str(xml.as_str()).unwrap())
    });
}

criterion_group!(benches, bulk_bench, criterion_benchmark);
criterion_main!(benches);
