use pine_builtin_macro::BuiltinFunction;
use pine_interpreter::{Interpreter, PineOutput, RuntimeError, SeriesBuffer, Value};

/// ta.stdev(source, length) - Standard Deviation
#[derive(BuiltinFunction)]
#[builtin(name = "ta.stdev", stateful)]
pub struct TaStdev {
    source: f64,
    length: f64,
    #[state]
    window: SeriesBuffer<f64>,
}

impl TaStdev {
    fn execute<O: PineOutput>(
        &mut self,
        _ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let length = self.length as usize;
        if length == 0 {
            return Err(RuntimeError::TypeError(
                "length must be greater than 0".to_string(),
            ));
        }

        let Some(values) = self.window.observe(self.source, length) else {
            return Ok(Value::Na);
        };

        if values.len() == 1 {
            return Ok(Value::Number(0.0));
        }

        // Calculate mean
        let mean: f64 = values.iter().sum::<f64>() / values.len() as f64;

        // Calculate variance
        let variance: f64 = values
            .iter()
            .map(|&val| {
                let diff = val - mean;
                diff * diff
            })
            .sum::<f64>()
            / values.len() as f64;

        // Standard deviation is square root of variance
        Ok(Value::Number(variance.sqrt()))
    }
}

/// ta.variance(source, length) - Variance
#[derive(BuiltinFunction)]
#[builtin(name = "ta.variance", stateful)]
pub struct TaVariance {
    source: f64,
    length: f64,
    #[state]
    window: SeriesBuffer<f64>,
}

impl TaVariance {
    fn execute<O: PineOutput>(
        &mut self,
        _ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let length = self.length as usize;
        if length == 0 {
            return Err(RuntimeError::TypeError(
                "length must be greater than 0".to_string(),
            ));
        }

        let Some(values) = self.window.observe(self.source, length) else {
            return Ok(Value::Na);
        };

        if values.len() == 1 {
            return Ok(Value::Number(0.0));
        }

        // Calculate mean
        let mean: f64 = values.iter().sum::<f64>() / values.len() as f64;

        // Calculate variance
        let variance: f64 = values
            .iter()
            .map(|&val| {
                let diff = val - mean;
                diff * diff
            })
            .sum::<f64>()
            / values.len() as f64;

        Ok(Value::Number(variance))
    }
}

/// ta.median(source, length) - Median value
#[derive(BuiltinFunction)]
#[builtin(name = "ta.median", stateful)]
pub struct TaMedian {
    source: f64,
    length: f64,
    #[state]
    window: SeriesBuffer<f64>,
}

impl TaMedian {
    fn execute<O: PineOutput>(
        &mut self,
        _ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let length = self.length as usize;
        if length == 0 {
            return Err(RuntimeError::TypeError(
                "length must be greater than 0".to_string(),
            ));
        }

        let Some(mut values) = self.window.observe(self.source, length) else {
            return Ok(Value::Na);
        };

        // Sort to find median
        values.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let mid = values.len() / 2;
        let median = if values.len() % 2 == 1 {
            // Odd number of elements - middle value
            values[mid]
        } else {
            // Even number of elements - average of two middle values
            (values[mid - 1] + values[mid]) / 2.0
        };

        Ok(Value::Number(median))
    }
}

/// ta.percentile_nearest_rank(source, length, percentage) - Percentile by the
/// nearest-rank method: the smallest value at or below which `percentage` of the
/// last `length` values fall.
#[derive(BuiltinFunction)]
#[builtin(name = "ta.percentile_nearest_rank", stateful)]
pub struct TaPercentileNearestRank {
    source: f64,
    length: f64,
    percentage: f64,
    #[state]
    window: SeriesBuffer<f64>,
}

impl TaPercentileNearestRank {
    fn execute<O: PineOutput>(
        &mut self,
        _ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let length = self.length as usize;
        if length == 0 {
            return Err(RuntimeError::TypeError(
                "length must be greater than 0".to_string(),
            ));
        }

        let Some(mut values) = self.window.observe(self.source, length) else {
            return Ok(Value::Na);
        };

        values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        // Nearest rank: n = ceil(P/100 * N), 1-based, clamped into the sample.
        let rank = (self.percentage / 100.0 * values.len() as f64).ceil() as usize;
        let index = rank.clamp(1, values.len()) - 1;

        Ok(Value::Number(values[index]))
    }
}

/// ta.cum(source) - Running total of `source` from the first bar onwards.
#[derive(BuiltinFunction)]
#[builtin(name = "ta.cum", stateful)]
pub struct TaCum {
    source: f64,
    /// The total so far, carried across bars by this call site.
    #[state]
    total: f64,
}

impl TaCum {
    fn execute<O: PineOutput>(
        &mut self,
        _ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        // Pine's `cum` skips na (which arrives as NaN) rather than letting it
        // poison the total for good.
        if self.source.is_finite() {
            self.total += self.source;
        }

        Ok(Value::Number(self.total))
    }
}

/// ta.dev(source, length) - Mean Absolute Deviation
#[derive(BuiltinFunction)]
#[builtin(name = "ta.dev", stateful)]
pub struct TaDev {
    source: f64,
    length: f64,
    #[state]
    window: SeriesBuffer<f64>,
}

impl TaDev {
    fn execute<O: PineOutput>(
        &mut self,
        _ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let length = self.length as usize;
        if length == 0 {
            return Err(RuntimeError::TypeError(
                "length must be greater than 0".to_string(),
            ));
        }

        let Some(values) = self.window.observe(self.source, length) else {
            return Ok(Value::Na);
        };

        // Calculate mean
        let mean: f64 = values.iter().sum::<f64>() / values.len() as f64;

        // Calculate mean absolute deviation
        let mad: f64 =
            values.iter().map(|&val| (val - mean).abs()).sum::<f64>() / values.len() as f64;

        Ok(Value::Number(mad))
    }
}
