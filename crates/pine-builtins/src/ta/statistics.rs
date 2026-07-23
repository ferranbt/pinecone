use pine_builtin_macro::BuiltinFunction;
use pine_interpreter::{Interpreter, PineOutput, RuntimeError, Value};

/// ta.stdev(source, length) - Standard Deviation
#[derive(BuiltinFunction)]
#[builtin(name = "ta.stdev")]
pub struct TaStdev<O: PineOutput> {
    source: Value<O>,
    length: f64,
}

impl<O: PineOutput> TaStdev<O> {
    fn execute(&self, ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let length = self.length as usize;
        if length == 0 {
            return Err(RuntimeError::TypeError(
                "length must be greater than 0".to_string(),
            ));
        }

        let values = ctx.get_series_values(&self.source, length)?;

        if values.is_empty() {
            return Ok(Value::Na);
        }

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
#[builtin(name = "ta.variance")]
pub struct TaVariance<O: PineOutput> {
    source: Value<O>,
    length: f64,
}

impl<O: PineOutput> TaVariance<O> {
    fn execute(&self, ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let length = self.length as usize;
        if length == 0 {
            return Err(RuntimeError::TypeError(
                "length must be greater than 0".to_string(),
            ));
        }

        let values = ctx.get_series_values(&self.source, length)?;

        if values.is_empty() {
            return Ok(Value::Na);
        }

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
#[builtin(name = "ta.median")]
pub struct TaMedian<O: PineOutput> {
    source: Value<O>,
    length: f64,
}

impl<O: PineOutput> TaMedian<O> {
    fn execute(&self, ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let length = self.length as usize;
        if length == 0 {
            return Err(RuntimeError::TypeError(
                "length must be greater than 0".to_string(),
            ));
        }

        let mut values = ctx.get_series_values(&self.source, length)?;

        if values.is_empty() {
            return Ok(Value::Na);
        }

        // Sort to find median
        values.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let median = if values.len() % 2 == 0 {
            // Even number of elements - average of two middle values
            let mid = values.len() / 2;
            (values[mid - 1] + values[mid]) / 2.0
        } else {
            // Odd number of elements - middle value
            values[values.len() / 2]
        };

        Ok(Value::Number(median))
    }
}

/// ta.percentile_nearest_rank(source, length, percentage) - Percentile by the
/// nearest-rank method: the smallest value at or below which `percentage` of the
/// last `length` values fall.
#[derive(BuiltinFunction)]
#[builtin(name = "ta.percentile_nearest_rank")]
pub struct TaPercentileNearestRank<O: PineOutput> {
    source: Value<O>,
    length: f64,
    percentage: f64,
}

impl<O: PineOutput> TaPercentileNearestRank<O> {
    fn execute(&self, ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let length = self.length as usize;
        if length == 0 {
            return Err(RuntimeError::TypeError(
                "length must be greater than 0".to_string(),
            ));
        }

        let mut values = ctx.get_series_values(&self.source, length)?;

        if values.is_empty() {
            return Ok(Value::Na);
        }

        values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        // Nearest rank: n = ceil(P/100 * N), 1-based, clamped into the sample.
        let rank = (self.percentage / 100.0 * values.len() as f64).ceil() as usize;
        let index = rank.clamp(1, values.len()) - 1;

        Ok(Value::Number(values[index]))
    }
}

/// ta.cum(source) - Running total of `source` from the first bar onwards.
#[derive(BuiltinFunction)]
#[builtin(name = "ta.cum")]
pub struct TaCum<O: PineOutput> {
    source: Value<O>,
}

impl<O: PineOutput> TaCum<O> {
    fn execute(&self, ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        // `usize::MAX` means "the whole history": `get_series_values` stops as
        // soon as the provider runs out of bars.
        let values = ctx.get_series_values(&self.source, usize::MAX)?;

        if values.is_empty() {
            return Ok(Value::Na);
        }

        // Pine's `cum` skips na rather than poisoning the total.
        Ok(Value::Number(
            values.iter().filter(|v| v.is_finite()).sum::<f64>(),
        ))
    }
}

/// ta.dev(source, length) - Mean Absolute Deviation
#[derive(BuiltinFunction)]
#[builtin(name = "ta.dev")]
pub struct TaDev<O: PineOutput> {
    source: Value<O>,
    length: f64,
}

impl<O: PineOutput> TaDev<O> {
    fn execute(&self, ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let length = self.length as usize;
        if length == 0 {
            return Err(RuntimeError::TypeError(
                "length must be greater than 0".to_string(),
            ));
        }

        let values = ctx.get_series_values(&self.source, length)?;

        if values.is_empty() {
            return Ok(Value::Na);
        }

        // Calculate mean
        let mean: f64 = values.iter().sum::<f64>() / values.len() as f64;

        // Calculate mean absolute deviation
        let mad: f64 =
            values.iter().map(|&val| (val - mean).abs()).sum::<f64>() / values.len() as f64;

        Ok(Value::Number(mad))
    }
}
