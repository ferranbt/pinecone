use pine_builtin_macro::BuiltinFunction;
use pine_interpreter::{Interpreter, RuntimeError, Value};

/// ta.stdev(source, length) - Standard Deviation
#[derive(BuiltinFunction)]
#[builtin(name = "ta.stdev")]
pub struct TaStdev {
    source: Value,
    length: f64,
}

impl TaStdev {
    fn execute(&self, ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
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
pub struct TaVariance {
    source: Value,
    length: f64,
}

impl TaVariance {
    fn execute(&self, ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
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
pub struct TaMedian {
    source: Value,
    length: f64,
}

impl TaMedian {
    fn execute(&self, ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
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

/// ta.dev(source, length) - Mean Absolute Deviation
#[derive(BuiltinFunction)]
#[builtin(name = "ta.dev")]
pub struct TaDev {
    source: Value,
    length: f64,
}

impl TaDev {
    fn execute(&self, ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
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
