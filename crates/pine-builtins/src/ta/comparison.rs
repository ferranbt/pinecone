use pine_builtin_macro::BuiltinFunction;
use pine_interpreter::{Interpreter, RuntimeError, Value};

/// ta.change(source, length) - Difference between current and N bars ago
#[derive(BuiltinFunction)]
#[builtin(name = "ta.change")]
pub struct TaChange {
    source: Value,
    #[arg(default = 1.0)]
    length: f64,
}

impl TaChange {
    fn execute(&self, ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let length = self.length as usize;

        if let Value::Series(series) = &self.source {
            let current = if let Value::Number(n) = *series.current {
                n
            } else {
                return Err(RuntimeError::TypeError(
                    "Series must contain numbers".to_string(),
                ));
            };

            if length == 0 {
                return Ok(Value::Number(0.0));
            }

            // Get value N bars ago
            if let Some(provider) = &ctx.historical_provider {
                if let Some(Value::Number(prev)) = provider.get_historical(&series.id, length) {
                    return Ok(Value::Number(current - prev));
                }
            }

            // Not enough data
            Ok(Value::Na)
        } else if let Value::Number(_) = self.source {
            // Single number has no change
            Ok(Value::Number(0.0))
        } else {
            Err(RuntimeError::TypeError(
                "source must be a number or series".to_string(),
            ))
        }
    }
}

/// ta.highest(source, length) - Highest value over N bars
#[derive(BuiltinFunction)]
#[builtin(name = "ta.highest")]
pub struct TaHighest {
    source: Value,
    length: f64,
}

impl TaHighest {
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

        let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        Ok(Value::Number(max))
    }
}

/// ta.lowest(source, length) - Lowest value over N bars
#[derive(BuiltinFunction)]
#[builtin(name = "ta.lowest")]
pub struct TaLowest {
    source: Value,
    length: f64,
}

impl TaLowest {
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

        let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
        Ok(Value::Number(min))
    }
}

/// ta.cross(source1, source2) - True if two series crossed
#[derive(BuiltinFunction)]
#[builtin(name = "ta.cross")]
pub struct TaCross {
    source1: Value,
    source2: Value,
}

impl TaCross {
    fn execute(&self, ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let vals1 = ctx.get_series_values(&self.source1, 2)?;
        let vals2 = ctx.get_series_values(&self.source2, 2)?;

        if vals1.len() < 2 || vals2.len() < 2 {
            return Ok(Value::Bool(false));
        }

        // Cross happens when (prev1 < prev2 && curr1 > curr2) || (prev1 > prev2 && curr1 < curr2)
        let crossed = (vals1[1] < vals2[1] && vals1[0] > vals2[0])
            || (vals1[1] > vals2[1] && vals1[0] < vals2[0]);

        Ok(Value::Bool(crossed))
    }
}

/// ta.crossover(source1, source2) - True if source1 crossed over source2
#[derive(BuiltinFunction)]
#[builtin(name = "ta.crossover")]
pub struct TaCrossover {
    source1: Value,
    source2: Value,
}

impl TaCrossover {
    fn execute(&self, ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let vals1 = ctx.get_series_values(&self.source1, 2)?;
        let vals2 = ctx.get_series_values(&self.source2, 2)?;

        if vals1.len() < 2 || vals2.len() < 2 {
            return Ok(Value::Bool(false));
        }

        // Crossover: prev1 <= prev2 && curr1 > curr2
        let crossed_over = vals1[1] <= vals2[1] && vals1[0] > vals2[0];

        Ok(Value::Bool(crossed_over))
    }
}

/// ta.crossunder(source1, source2) - True if source1 crossed under source2
#[derive(BuiltinFunction)]
#[builtin(name = "ta.crossunder")]
pub struct TaCrossunder {
    source1: Value,
    source2: Value,
}

impl TaCrossunder {
    fn execute(&self, ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let vals1 = ctx.get_series_values(&self.source1, 2)?;
        let vals2 = ctx.get_series_values(&self.source2, 2)?;

        if vals1.len() < 2 || vals2.len() < 2 {
            return Ok(Value::Bool(false));
        }

        // Crossunder: prev1 >= prev2 && curr1 < curr2
        let crossed_under = vals1[1] >= vals2[1] && vals1[0] < vals2[0];

        Ok(Value::Bool(crossed_under))
    }
}

/// ta.rising(source, length) - Test if source is rising for length bars
#[derive(BuiltinFunction)]
#[builtin(name = "ta.rising")]
pub struct TaRising {
    source: Value,
    length: f64,
}

impl TaRising {
    fn execute(&self, ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let length = self.length as usize;
        if length == 0 {
            return Ok(Value::Bool(false));
        }

        let values = ctx.get_series_values(&self.source, length + 1)?;

        if values.len() <= length {
            return Ok(Value::Bool(false));
        }

        // Check if current value is greater than all previous values
        let current = values[0];
        for value in values.iter().take(length + 1).skip(1) {
            if current <= *value {
                return Ok(Value::Bool(false));
            }
        }

        Ok(Value::Bool(true))
    }
}

/// ta.falling(source, length) - Test if source is falling for length bars
#[derive(BuiltinFunction)]
#[builtin(name = "ta.falling")]
pub struct TaFalling {
    source: Value,
    length: f64,
}

impl TaFalling {
    fn execute(&self, ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let length = self.length as usize;
        if length == 0 {
            return Ok(Value::Bool(false));
        }

        let values = ctx.get_series_values(&self.source, length + 1)?;

        if values.len() <= length {
            return Ok(Value::Bool(false));
        }

        // Check if current value is less than all previous values
        let current = values[0];
        for value in values.iter().take(length + 1).skip(1) {
            if current >= *value {
                return Ok(Value::Bool(false));
            }
        }

        Ok(Value::Bool(true))
    }
}

/// ta.highestbars(source, length) - Offset to highest value (0 = current bar)
#[derive(BuiltinFunction)]
#[builtin(name = "ta.highestbars")]
pub struct TaHighestbars {
    source: Value,
    length: f64,
}

impl TaHighestbars {
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

        // Find index of maximum value
        let mut max_idx = 0;
        let mut max_val = values[0];

        for (i, &val) in values.iter().enumerate() {
            if val > max_val {
                max_val = val;
                max_idx = i;
            }
        }

        // Return negative offset (0 = current, -1 = 1 bar ago, etc.)
        Ok(Value::Number(-(max_idx as f64)))
    }
}

/// ta.lowestbars(source, length) - Offset to lowest value (0 = current bar)
#[derive(BuiltinFunction)]
#[builtin(name = "ta.lowestbars")]
pub struct TaLowestbars {
    source: Value,
    length: f64,
}

impl TaLowestbars {
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

        // Find index of minimum value
        let mut min_idx = 0;
        let mut min_val = values[0];

        for (i, &val) in values.iter().enumerate() {
            if val < min_val {
                min_val = val;
                min_idx = i;
            }
        }

        // Return negative offset (0 = current, -1 = 1 bar ago, etc.)
        Ok(Value::Number(-(min_idx as f64)))
    }
}
