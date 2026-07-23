use pine::ScriptBuilder;
use pine_builtins::DefaultPineOutput;
use pine_interpreter::Bar;

/// Generate synthetic OHLCV bar data for benchmarking
pub fn generate_bars(count: usize) -> Vec<Bar> {
    let mut bars = Vec::with_capacity(count);
    let mut price = 100.0;

    for _ in 0..count {
        // Simulate random price movement
        let change = (price * 0.02 * ((bars.len() as f64 * 17.0).sin()))
            .max(-price * 0.05)
            .min(price * 0.05);
        price += change;

        let open = price;
        let high = price * (1.0 + 0.01 * ((bars.len() as f64 * 13.0).cos()).abs());
        let low = price * (1.0 - 0.01 * ((bars.len() as f64 * 19.0).sin()).abs());
        let close = price + (high - low) * 0.3;
        let volume = 1000000.0 + 500000.0 * ((bars.len() as f64 * 7.0).sin()).abs();

        bars.push(Bar {
            open,
            high,
            low,
            close,
            volume,
            ..Default::default()
        });

        price = close;
    }

    bars
}

/// Compile `source` and run it over every bar.
///
/// Series history and stateful builtins accumulate as bars execute, so a script
/// that looks back has to be replayed from the start rather than evaluated at a
/// single bar. This measures the whole replay.
pub fn execute_with_history(source: &str, bars: &[Bar]) -> Result<(), pine::Error> {
    let mut script = ScriptBuilder::<DefaultPineOutput>::with_code(source).compile()?;
    script.execute_bars(bars)
}
