use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use pine_core::MAX_LOOKBACK;
use pine_lang::execute;
use pinecone_benches::{generate_bars, TEST_SCRIPTS};

fn bench_full_run(c: &mut Criterion) {
    let mut group = c.benchmark_group("interpreter/single_bar");
    let data = generate_bars(MAX_LOOKBACK * 2);

    for (name, source) in TEST_SCRIPTS {
        group.bench_with_input(BenchmarkId::from_parameter(name), source, |b, source| {
            b.iter(|| {
                execute(black_box(source), data.clone()).unwrap();
            });
        });
    }

    group.finish();
}

criterion_group!(benches, bench_full_run);
criterion_main!(benches);
