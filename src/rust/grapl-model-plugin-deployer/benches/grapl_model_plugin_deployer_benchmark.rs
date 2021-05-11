use criterion::{
    criterion_group,
    criterion_main,
    Criterion,
};

pub fn criterion_benchmark(_c: &mut Criterion) {
    todo!("Add some benchmarks");
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
