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
        for i in 1..=length {
            if current <= values[i] {
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
        for i in 1..=length {
            if current >= values[i] {
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

#[cfg(test)]
mod tests {
    use super::*;
    use pine_interpreter::{EvaluatedArg, HistoricalDataProvider, Series};
    use std::collections::HashMap;

    struct MockHistoricalData {
        data: HashMap<String, Vec<f64>>,
    }

    impl HistoricalDataProvider for MockHistoricalData {
        fn get_historical(&self, series_id: &str, offset: usize) -> Option<Value> {
            self.data
                .get(series_id)
                .and_then(|values| values.get(offset - 1))
                .map(|&v| Value::Number(v))
        }
    }

    #[test]
    fn test_ta_change() {
        let mut ctx = Interpreter::new();

        let mut data = HashMap::new();
        data.insert("close".to_string(), vec![100.0, 95.0, 90.0]);
        ctx.set_historical_provider(Box::new(MockHistoricalData { data }));

        let series = Value::Series(Series {
            id: "close".to_string(),
            current: Box::new(Value::Number(105.0)),
        });

        // change(close, 1) = 105 - 100 = 5
        let result = TaChange::builtin_fn(
            &mut ctx,
            vec![
                EvaluatedArg::Positional(series.clone()),
                EvaluatedArg::Positional(Value::Number(1.0)),
            ],
        )
        .unwrap();
        assert_eq!(result, Value::Number(5.0));

        // change(close, 2) = 105 - 95 = 10
        let result = TaChange::builtin_fn(
            &mut ctx,
            vec![
                EvaluatedArg::Positional(series),
                EvaluatedArg::Positional(Value::Number(2.0)),
            ],
        )
        .unwrap();
        assert_eq!(result, Value::Number(10.0));
    }

    #[test]
    fn test_ta_highest_lowest() {
        let mut ctx = Interpreter::new();

        let mut data = HashMap::new();
        data.insert("close".to_string(), vec![102.0, 98.0, 105.0, 100.0]);
        ctx.set_historical_provider(Box::new(MockHistoricalData { data }));

        let series = Value::Series(Series {
            id: "close".to_string(),
            current: Box::new(Value::Number(103.0)),
        });

        // highest over 5 bars: max(103, 102, 98, 105, 100) = 105
        let result = TaHighest::builtin_fn(
            &mut ctx,
            vec![
                EvaluatedArg::Positional(series.clone()),
                EvaluatedArg::Positional(Value::Number(5.0)),
            ],
        )
        .unwrap();
        assert_eq!(result, Value::Number(105.0));

        // lowest over 5 bars: min(103, 102, 98, 105, 100) = 98
        let result = TaLowest::builtin_fn(
            &mut ctx,
            vec![
                EvaluatedArg::Positional(series),
                EvaluatedArg::Positional(Value::Number(5.0)),
            ],
        )
        .unwrap();
        assert_eq!(result, Value::Number(98.0));
    }

    #[test]
    fn test_ta_cross() {
        let mut ctx = Interpreter::new();

        let mut data = HashMap::new();
        data.insert("fast".to_string(), vec![95.0]);
        data.insert("slow".to_string(), vec![105.0]);
        ctx.set_historical_provider(Box::new(MockHistoricalData { data }));

        let fast = Value::Series(Series {
            id: "fast".to_string(),
            current: Box::new(Value::Number(110.0)),
        });

        let slow = Value::Series(Series {
            id: "slow".to_string(),
            current: Box::new(Value::Number(100.0)),
        });

        // fast crossed slow: prev fast(95) < prev slow(105), curr fast(110) > curr slow(100)
        let result = TaCross::builtin_fn(
            &mut ctx,
            vec![
                EvaluatedArg::Positional(fast),
                EvaluatedArg::Positional(slow),
            ],
        )
        .unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_ta_crossover() {
        let mut ctx = Interpreter::new();

        let mut data = HashMap::new();
        data.insert("fast".to_string(), vec![95.0]);
        data.insert("slow".to_string(), vec![100.0]);
        ctx.set_historical_provider(Box::new(MockHistoricalData { data }));

        let fast = Value::Series(Series {
            id: "fast".to_string(),
            current: Box::new(Value::Number(105.0)),
        });

        let slow = Value::Series(Series {
            id: "slow".to_string(),
            current: Box::new(Value::Number(100.0)),
        });

        // Crossover: prev fast(95) <= prev slow(100), curr fast(105) > curr slow(100)
        let result = TaCrossover::builtin_fn(
            &mut ctx,
            vec![
                EvaluatedArg::Positional(fast),
                EvaluatedArg::Positional(slow),
            ],
        )
        .unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_ta_crossunder() {
        let mut ctx = Interpreter::new();

        let mut data = HashMap::new();
        data.insert("fast".to_string(), vec![105.0]);
        data.insert("slow".to_string(), vec![100.0]);
        ctx.set_historical_provider(Box::new(MockHistoricalData { data }));

        let fast = Value::Series(Series {
            id: "fast".to_string(),
            current: Box::new(Value::Number(95.0)),
        });

        let slow = Value::Series(Series {
            id: "slow".to_string(),
            current: Box::new(Value::Number(100.0)),
        });

        // Crossunder: prev fast(105) >= prev slow(100), curr fast(95) < curr slow(100)
        let result = TaCrossunder::builtin_fn(
            &mut ctx,
            vec![
                EvaluatedArg::Positional(fast),
                EvaluatedArg::Positional(slow),
            ],
        )
        .unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_ta_rising() {
        let mut ctx = Interpreter::new();

        let mut data = HashMap::new();
        data.insert("close".to_string(), vec![92.0, 90.0, 85.0]);
        ctx.set_historical_provider(Box::new(MockHistoricalData { data }));

        let series = Value::Series(Series {
            id: "close".to_string(),
            current: Box::new(Value::Number(95.0)),
        });

        // rising(3): 95 > 92 > 90 > 85 = true
        let result = TaRising::builtin_fn(
            &mut ctx,
            vec![
                EvaluatedArg::Positional(series),
                EvaluatedArg::Positional(Value::Number(3.0)),
            ],
        )
        .unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_ta_falling() {
        let mut ctx = Interpreter::new();

        let mut data = HashMap::new();
        data.insert("close".to_string(), vec![92.0, 95.0, 98.0]);
        ctx.set_historical_provider(Box::new(MockHistoricalData { data }));

        let series = Value::Series(Series {
            id: "close".to_string(),
            current: Box::new(Value::Number(90.0)),
        });

        // falling(3): 90 < 92 < 95 < 98 = true
        let result = TaFalling::builtin_fn(
            &mut ctx,
            vec![
                EvaluatedArg::Positional(series),
                EvaluatedArg::Positional(Value::Number(3.0)),
            ],
        )
        .unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_ta_highestbars() {
        let mut ctx = Interpreter::new();

        let mut data = HashMap::new();
        data.insert("close".to_string(), vec![102.0, 110.0, 105.0, 100.0]);
        ctx.set_historical_provider(Box::new(MockHistoricalData { data }));

        let series = Value::Series(Series {
            id: "close".to_string(),
            current: Box::new(Value::Number(103.0)),
        });

        // Values: [103, 102, 110, 105, 100] - highest is 110 at index 2 (2 bars ago)
        let result = TaHighestbars::builtin_fn(
            &mut ctx,
            vec![
                EvaluatedArg::Positional(series),
                EvaluatedArg::Positional(Value::Number(5.0)),
            ],
        )
        .unwrap();
        assert_eq!(result, Value::Number(-2.0));
    }

    #[test]
    fn test_ta_lowestbars() {
        let mut ctx = Interpreter::new();

        let mut data = HashMap::new();
        data.insert("close".to_string(), vec![102.0, 95.0, 105.0, 100.0]);
        ctx.set_historical_provider(Box::new(MockHistoricalData { data }));

        let series = Value::Series(Series {
            id: "close".to_string(),
            current: Box::new(Value::Number(103.0)),
        });

        // Values: [103, 102, 95, 105, 100] - lowest is 95 at index 2 (2 bars ago)
        let result = TaLowestbars::builtin_fn(
            &mut ctx,
            vec![
                EvaluatedArg::Positional(series),
                EvaluatedArg::Positional(Value::Number(5.0)),
            ],
        )
        .unwrap();
        assert_eq!(result, Value::Number(-2.0));
    }
}
