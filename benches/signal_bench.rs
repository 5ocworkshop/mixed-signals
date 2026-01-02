use criterion::{black_box, criterion_group, criterion_main, Criterion};
use mixed_signals::prelude::*;

fn bench_sine_sample(c: &mut Criterion) {
    let sine = Sine::default();
    c.bench_function("sine_sample", |b| {
        b.iter(|| {
            let mut acc = 0.0;
            for i in 0..1024 {
                acc += sine.sample(i as f64 * 0.001);
            }
            black_box(acc);
        })
    });
}

fn bench_sine_sample_into(c: &mut Criterion) {
    let sine = Sine::default();
    let mut buffer = vec![0.0f32; 1024];
    c.bench_function("sine_sample_into", |b| {
        b.iter(|| {
            sine.sample_into(0.0, 0.001, &mut buffer);
            black_box(buffer[0]);
        })
    });
}

fn bench_mix_sample(c: &mut Criterion) {
    let a = Sine::with_frequency(1.0);
    let b = Triangle::with_frequency(0.5);
    let mix = Mix::new(a, b, 0.35);
    c.bench_function("mix_sample", |b| {
        b.iter(|| {
            let mut acc = 0.0;
            for i in 0..1024 {
                acc += mix.sample(i as f64 * 0.001);
            }
            black_box(acc);
        })
    });
}

criterion_group!(
    signal_benches,
    bench_sine_sample,
    bench_sine_sample_into,
    bench_mix_sample
);
criterion_main!(signal_benches);
