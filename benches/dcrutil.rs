use criterion::{criterion_group, criterion_main, Criterion};

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("dcrutil::app_data_dir", |b| {
        b.iter(|| {
            let _ = dcrdrs::dcrutil::appdata::app_data_dir(&mut "dcrd".into(), false);
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
