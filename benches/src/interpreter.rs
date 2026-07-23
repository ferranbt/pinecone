mod test_data;

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use pine::interpreter::DefaultPineOutput;
use pine::{execute, ScriptBuilder};
use test_data::generate_bars;

const TEST_SCRIPTS: &[(&str, &str)] = &[
    ("simple", include_str!("../test_data/simple.pine")),
    (
        "moving_averages",
        include_str!("../test_data/moving_averages.pine"),
    ),
    ("rsi", include_str!("../test_data/rsi.pine")),
    ("macd", include_str!("../test_data/macd.pine")),
    ("complex", include_str!("../test_data/complex.pine")),
];

fn bench_single_bar(c: &mut Criterion) {
    let mut group = c.benchmark_group("interpreter/single_bar");
    let data = generate_bars(200); // Generate enough bars for historical lookback

    for (name, source) in TEST_SCRIPTS {
        group.bench_with_input(BenchmarkId::from_parameter(name), source, |b, source| {
            b.iter(|| {
                execute(black_box(source), data.clone()).unwrap();
            });
        });
    }

    group.finish();
}

fn bench_compile_only(c: &mut Criterion) {
    let mut group = c.benchmark_group("interpreter/compile");

    for (name, source) in TEST_SCRIPTS {
        group.bench_with_input(BenchmarkId::from_parameter(name), source, |b, source| {
            b.iter(|| {
                let _ = ScriptBuilder::<DefaultPineOutput>::with_code(black_box(source))
                    .compile()
                    .unwrap();
            });
        });
    }

    group.finish();
}

criterion_group!(benches, bench_compile_only, bench_single_bar);
criterion_main!(benches);
