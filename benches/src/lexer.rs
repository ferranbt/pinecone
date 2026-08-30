use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use pine_lexer::Lexer;
use pinecone_benches::TEST_SCRIPTS;

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
