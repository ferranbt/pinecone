mod test_data;

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use test_data::{execute_with_history, generate_bars};

fn bench_moving_averages(c: &mut Criterion) {
    let mut group = c.benchmark_group("ta/moving_averages");

    let scripts = [
        ("sma", "sma20 = ta.sma(close, 20)"),
        ("ema", "ema20 = ta.ema(close, 20)"),
        ("wma", "wma20 = ta.wma(close, 20)"),
        ("rma", "rma20 = ta.rma(close, 20)"),
        ("hma", "hma20 = ta.hma(close, 20)"),
    ];

    for lookback in [50, 100, 200].iter() {
        let bars = generate_bars(*lookback);

        for (name, source) in scripts.iter() {
            group.bench_with_input(
                BenchmarkId::new(*name, lookback),
                source,
                |b, source| {
                    b.iter(|| {
                        execute_with_history(black_box(source), &bars).unwrap();
                    });
                },
            );
        }
    }

    group.finish();
}

fn bench_oscillators(c: &mut Criterion) {
    let mut group = c.benchmark_group("ta/oscillators");

    let scripts = [
        ("rsi", "rsi14 = ta.rsi(close, 14)"),
        ("cci", "cci20 = ta.cci(close, 20)"),
        ("mom", "mom10 = ta.mom(close, 10)"),
        ("roc", "roc12 = ta.roc(close, 12)"),
        ("cmo", "cmo14 = ta.cmo(close, 14)"),
    ];

    for lookback in [50, 100, 200].iter() {
        let bars = generate_bars(*lookback);

        for (name, source) in scripts.iter() {
            group.bench_with_input(
                BenchmarkId::new(*name, lookback),
                source,
                |b, source| {
                    b.iter(|| {
                        execute_with_history(black_box(source), &bars).unwrap();
                    });
                },
            );
        }
    }

    group.finish();
}

fn bench_volatility(c: &mut Criterion) {
    let mut group = c.benchmark_group("ta/volatility");

    let scripts = [
        ("atr", "atr14 = ta.atr(14)"),
        ("stdev", "stdev20 = ta.stdev(close, 20)"),
        ("tr", "tr_val = ta.tr(true)"),
    ];

    for lookback in [50, 100, 200].iter() {
        let bars = generate_bars(*lookback);

        for (name, source) in scripts.iter() {
            group.bench_with_input(
                BenchmarkId::new(*name, lookback),
                source,
                |b, source| {
                    b.iter(|| {
                        execute_with_history(black_box(source), &bars).unwrap();
                    });
                },
            );
        }
    }

    group.finish();
}

fn bench_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("ta/comparison");

    let scripts = [
        ("cross", "crossed = ta.cross(close, ta.sma(close, 20))"),
        ("crossover", "co = ta.crossover(close, ta.sma(close, 20))"),
        ("crossunder", "cu = ta.crossunder(close, ta.sma(close, 20))"),
        ("rising", "up = ta.rising(close, 3)"),
        ("falling", "down = ta.falling(close, 3)"),
    ];

    for lookback in [50, 100, 200].iter() {
        let bars = generate_bars(*lookback);

        for (name, source) in scripts.iter() {
            group.bench_with_input(
                BenchmarkId::new(*name, lookback),
                source,
                |b, source| {
                    b.iter(|| {
                        execute_with_history(black_box(source), &bars).unwrap();
                    });
                },
            );
        }
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_moving_averages,
    bench_oscillators,
    bench_volatility,
    bench_comparison
);
criterion_main!(benches);
