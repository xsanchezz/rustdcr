use criterion::criterion_main;

mod dcrutil;

criterion_main!(dcrutil::app_data::app_data_dir);
