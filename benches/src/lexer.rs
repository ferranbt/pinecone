use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use pine_lexer::Lexer;

const TEST_SCRIPTS: &[(&str, &str)] = &[
    ("simple", include_str!("../test_data/simple.pine")),
    ("moving_averages", include_str!("../test_data/moving_averages.pine")),
    ("rsi", include_str!("../test_data/rsi.pine")),
    ("macd", include_str!("../test_data/macd.pine")),
    ("complex", include_str!("../test_data/complex.pine")),
];

fn bench_lexer(c: &mut Criterion) {
    let mut group = c.benchmark_group("lexer");

    for (name, source) in TEST_SCRIPTS {
        group.bench_with_input(BenchmarkId::new("tokenize", name), source, |b, source| {
            b.iter(|| {
                let mut lexer = Lexer::new(black_box(source));
                let _ = lexer.tokenize();
            });
        });
    }

    group.finish();
}

criterion_group!(benches, bench_lexer);
criterion_main!(benches);
