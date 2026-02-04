use pine::Script;
use pine_interpreter::{Bar, HistoricalDataProvider, Value};
use std::cell::Cell;

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
        });

        price = close;
    }

    bars
}

/// Mock historical data provider for benchmarks
pub struct BenchHistoricalData {
    bars: Vec<Bar>,
    current_index: Cell<usize>,
}

impl BenchHistoricalData {
    pub fn new(bars: Vec<Bar>) -> Self {
        Self {
            bars,
            current_index: Cell::new(0),
        }
    }

    pub fn set_current_bar(&self, index: usize) {
        self.current_index.set(index);
    }
}

impl HistoricalDataProvider for BenchHistoricalData {
    fn get_historical(&self, series_id: &str, offset: usize) -> Option<Value> {
        let current_index = self.current_index.get();

        if current_index < offset {
            return None;
        }

        let bar_index = current_index - offset;
        if bar_index >= self.bars.len() {
            return None;
        }

        let bar = &self.bars[bar_index];
        let value = match series_id {
            "open" => bar.open,
            "high" => bar.high,
            "low" => bar.low,
            "close" => bar.close,
            "volume" => bar.volume,
            _ => return None,
        };

        Some(Value::Number(value))
    }
}

/// Execute a script with historical data context
///
/// This helper compiles a script, sets up the historical data provider,
/// and executes it with the last bar in the dataset.
pub fn execute_with_history(source: &str, bars: &[Bar]) -> Result<(), pine::Error> {
    let mut script = Script::compile::<pine_builtins::DefaultLogger>(source, None)?;
    let historical_data = BenchHistoricalData::new(bars.to_vec());
    historical_data.set_current_bar(bars.len() - 1);
    script.set_historical_provider(Box::new(historical_data));
    script.execute(&bars[bars.len() - 1])?;
    Ok(())
}
