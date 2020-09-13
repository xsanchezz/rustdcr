use criterion::{criterion_group, criterion_main, Criterion};

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("dcrutil::app_data_dir", |b| {
        let mut path = String::from("dcrd");
        b.iter(|| dcrdrs::dcrutil::appdata::app_data_dir(&mut path, false))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
