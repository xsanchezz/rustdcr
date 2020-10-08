use criterion::{criterion_group, Criterion};

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("dcrutil::app_data_dir", |b| {
        b.iter(|| {
            let _ = rustdcr::dcrutil::get_app_data_dir("dcrd", false);
        })
    });
}

criterion_group!(app_data_dir, criterion_benchmark);
