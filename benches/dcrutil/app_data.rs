use criterion::{criterion_group, Criterion};

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("dcrutil::app_data_dir", |b| {
        b.iter(|| {
            let _ = dcrdrs::dcrutil::app_data::app_data_dir(&mut "dcrd".into(), false);
        })
    });
}

criterion_group!(app_data_dir, criterion_benchmark);
