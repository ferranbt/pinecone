use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use pine_lexer::Lexer;
use pine_parser::Parser;

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

fn bench_parser(c: &mut Criterion) {
    let mut group = c.benchmark_group("parser");

    for (name, source) in TEST_SCRIPTS {
        group.bench_with_input(BenchmarkId::new("parse", name), source, |b, source| {
            b.iter(|| {
                let mut lexer = Lexer::new(black_box(source));
                let tokens = lexer.tokenize().unwrap();
                let mut parser = Parser::new(tokens);
                let _ = parser.parse();
            });
        });
    }

    group.finish();
}

criterion_group!(benches, bench_parser);
criterion_main!(benches);
