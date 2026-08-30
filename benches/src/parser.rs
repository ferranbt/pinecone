use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use pine_lexer::Lexer;
use pine_parser::Parser;
use pinecone_benches::TEST_SCRIPTS;

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
