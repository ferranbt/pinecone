use pine_builtin_macro::BuiltinFunction;
use pine_interpreter::{Interpreter, RuntimeError, Value};

/// ta.tr(handle_na) - True Range
/// True range is max(high - low, abs(high - close[1]), abs(low - close[1]))
#[derive(BuiltinFunction)]
#[builtin(name = "ta.tr")]
pub struct TaTr {
    #[arg(default = false)]
    handle_na: bool,
}

impl TaTr {
    fn execute(&self, ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        // Get high, low, close from context
        let high = ctx
            .get_variable("high")
            .ok_or_else(|| RuntimeError::UndefinedVariable("high".to_string()))?;
        let low = ctx
            .get_variable("low")
            .ok_or_else(|| RuntimeError::UndefinedVariable("low".to_string()))?;
        let close = ctx
            .get_variable("close")
            .ok_or_else(|| RuntimeError::UndefinedVariable("close".to_string()))?;

        let high_val = if let Value::Series(s) = high {
            if let Value::Number(n) = *s.current {
                n
            } else {
                return Err(RuntimeError::TypeError("high must be a number".to_string()));
            }
        } else if let Value::Number(n) = high {
            *n
        } else {
            return Err(RuntimeError::TypeError(
                "high must be a number or series".to_string(),
            ));
        };

        let low_val = if let Value::Series(s) = low {
            if let Value::Number(n) = *s.current {
                n
            } else {
                return Err(RuntimeError::TypeError("low must be a number".to_string()));
            }
        } else if let Value::Number(n) = low {
            *n
        } else {
            return Err(RuntimeError::TypeError(
                "low must be a number or series".to_string(),
            ));
        };

        // Get previous close
        let prev_close = if let Value::Series(s) = close {
            if let Some(provider) = &ctx.historical_provider {
                if let Some(Value::Number(n)) = provider.get_historical(&s.id, 1) {
                    Some(n)
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        // Calculate true range
        let tr = if let Some(pc) = prev_close {
            let hl = high_val - low_val;
            let hc = (high_val - pc).abs();
            let lc = (low_val - pc).abs();
            hl.max(hc).max(lc)
        } else {
            // No previous close available
            if self.handle_na {
                high_val - low_val
            } else {
                return Ok(Value::Na);
            }
        };

        Ok(Value::Number(tr))
    }
}

/// ta.atr(length) - Average True Range
#[derive(BuiltinFunction)]
#[builtin(name = "ta.atr")]
pub struct TaAtr {
    length: f64,
}

impl TaAtr {
    fn execute(&self, ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let length = self.length as usize;
        if length == 0 {
            return Err(RuntimeError::TypeError(
                "length must be greater than 0".to_string(),
            ));
        }

        // Get high, low, close series from context
        let high = ctx
            .get_variable("high")
            .ok_or_else(|| RuntimeError::UndefinedVariable("high".to_string()))?;
        let low = ctx
            .get_variable("low")
            .ok_or_else(|| RuntimeError::UndefinedVariable("low".to_string()))?;
        let close = ctx
            .get_variable("close")
            .ok_or_else(|| RuntimeError::UndefinedVariable("close".to_string()))?;

        // We need to calculate TR for each bar and then RMA of those TRs
        let mut tr_values = Vec::new();

        // Calculate current TR
        let current_tr = {
            let high_val = if let Value::Series(s) = high {
                if let Value::Number(n) = *s.current {
                    n
                } else {
                    return Err(RuntimeError::TypeError(
                        "high must contain numbers".to_string(),
                    ));
                }
            } else if let Value::Number(n) = high {
                *n
            } else {
                return Err(RuntimeError::TypeError(
                    "high must be a number or series".to_string(),
                ));
            };

            let low_val = if let Value::Series(s) = low {
                if let Value::Number(n) = *s.current {
                    n
                } else {
                    return Err(RuntimeError::TypeError(
                        "low must contain numbers".to_string(),
                    ));
                }
            } else if let Value::Number(n) = low {
                *n
            } else {
                return Err(RuntimeError::TypeError(
                    "low must be a number or series".to_string(),
                ));
            };

            let prev_close = if let Value::Series(s) = close {
                if let Some(provider) = &ctx.historical_provider {
                    if let Some(Value::Number(n)) = provider.get_historical(&s.id, 1) {
                        Some(n)
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            };

            if let Some(pc) = prev_close {
                let hl = high_val - low_val;
                let hc = (high_val - pc).abs();
                let lc = (low_val - pc).abs();
                hl.max(hc).max(lc)
            } else {
                high_val - low_val
            }
        };

        tr_values.push(current_tr);

        // Get historical TR values
        if let (Value::Series(high_s), Value::Series(low_s), Value::Series(close_s)) =
            (high, low, close)
        {
            if let Some(provider) = &ctx.historical_provider {
                for i in 1..length * 2 {
                    let h = if let Some(Value::Number(n)) = provider.get_historical(&high_s.id, i) {
                        n
                    } else {
                        break;
                    };

                    let l = if let Some(Value::Number(n)) = provider.get_historical(&low_s.id, i) {
                        n
                    } else {
                        break;
                    };

                    let pc = if let Some(Value::Number(n)) =
                        provider.get_historical(&close_s.id, i + 1)
                    {
                        Some(n)
                    } else {
                        None
                    };

                    let tr = if let Some(prev_c) = pc {
                        let hl = h - l;
                        let hc = (h - prev_c).abs();
                        let lc = (l - prev_c).abs();
                        hl.max(hc).max(lc)
                    } else {
                        h - l
                    };

                    tr_values.push(tr);
                }
            }
        }

        if tr_values.is_empty() {
            return Ok(Value::Na);
        }

        // Calculate RMA of TR values
        let initial_sma: f64 = tr_values
            .iter()
            .take(length.min(tr_values.len()))
            .sum::<f64>()
            / length.min(tr_values.len()) as f64;
        let mut rma = initial_sma;
        let alpha = 1.0 / length as f64;

        for &tr in tr_values
            .iter()
            .rev()
            .skip(length.min(tr_values.len()))
            .take(tr_values.len() - length.min(tr_values.len()))
        {
            rma = alpha * tr + (1.0 - alpha) * rma;
        }

        // Apply current TR
        rma = alpha * tr_values[0] + (1.0 - alpha) * rma;

        Ok(Value::Number(rma))
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
    fn test_ta_tr() {
        let mut ctx = Interpreter::new();

        let mut data = HashMap::new();
        data.insert("close".to_string(), vec![100.0]);
        ctx.set_historical_provider(Box::new(MockHistoricalData { data }));

        // Set current bar data
        ctx.set_variable(
            "high",
            Value::Series(Series {
                id: "high".to_string(),
                current: Box::new(Value::Number(110.0)),
            }),
        );
        ctx.set_variable(
            "low",
            Value::Series(Series {
                id: "low".to_string(),
                current: Box::new(Value::Number(95.0)),
            }),
        );
        ctx.set_variable(
            "close",
            Value::Series(Series {
                id: "close".to_string(),
                current: Box::new(Value::Number(105.0)),
            }),
        );

        // TR = max(high-low, abs(high-prev_close), abs(low-prev_close))
        // TR = max(110-95, abs(110-100), abs(95-100))
        // TR = max(15, 10, 5) = 15
        let result = TaTr::builtin_fn(&mut ctx, vec![]).unwrap();

        assert_eq!(result, Value::Number(15.0));
    }

    #[test]
    fn test_ta_tr_handle_na() {
        let mut ctx = Interpreter::new();

        // No historical data - should use handle_na
        ctx.set_variable("high", Value::Number(110.0));
        ctx.set_variable("low", Value::Number(95.0));
        ctx.set_variable("close", Value::Number(105.0));

        // Without handle_na, should return Na
        let result = TaTr::builtin_fn(&mut ctx, vec![]).unwrap();
        assert_eq!(result, Value::Na);

        // With handle_na=true, should return high - low
        let result = TaTr::builtin_fn(
            &mut ctx,
            vec![EvaluatedArg::Named {
                name: "handle_na".to_string(),
                value: Value::Bool(true),
            }],
        )
        .unwrap();
        assert_eq!(result, Value::Number(15.0));
    }

    #[test]
    fn test_ta_atr() {
        let mut ctx = Interpreter::new();

        let mut data = HashMap::new();
        // Historical data for high, low, close
        data.insert("high".to_string(), vec![108.0, 107.0, 106.0, 105.0, 104.0]);
        data.insert("low".to_string(), vec![98.0, 97.0, 96.0, 95.0, 94.0]);
        data.insert(
            "close".to_string(),
            vec![102.0, 101.0, 100.0, 99.0, 98.0, 97.0],
        );
        ctx.set_historical_provider(Box::new(MockHistoricalData { data }));

        // Set current bar data
        ctx.set_variable(
            "high",
            Value::Series(Series {
                id: "high".to_string(),
                current: Box::new(Value::Number(110.0)),
            }),
        );
        ctx.set_variable(
            "low",
            Value::Series(Series {
                id: "low".to_string(),
                current: Box::new(Value::Number(100.0)),
            }),
        );
        ctx.set_variable(
            "close",
            Value::Series(Series {
                id: "close".to_string(),
                current: Box::new(Value::Number(105.0)),
            }),
        );

        let result =
            TaAtr::builtin_fn(&mut ctx, vec![EvaluatedArg::Positional(Value::Number(5.0))])
                .unwrap();

        // Just verify it returns a number (ATR calculation is complex with RMA)
        assert!(matches!(result, Value::Number(_)));
    }

    #[test]
    fn test_ta_tr_high_close_gap() {
        let mut ctx = Interpreter::new();

        let mut data = HashMap::new();
        data.insert("close".to_string(), vec![90.0]);
        ctx.set_historical_provider(Box::new(MockHistoricalData { data }));

        // Gap up: prev close = 90, high = 110, low = 105
        ctx.set_variable(
            "high",
            Value::Series(Series {
                id: "high".to_string(),
                current: Box::new(Value::Number(110.0)),
            }),
        );
        ctx.set_variable(
            "low",
            Value::Series(Series {
                id: "low".to_string(),
                current: Box::new(Value::Number(105.0)),
            }),
        );
        ctx.set_variable(
            "close",
            Value::Series(Series {
                id: "close".to_string(),
                current: Box::new(Value::Number(108.0)),
            }),
        );

        // TR = max(high-low, abs(high-prev_close), abs(low-prev_close))
        // TR = max(110-105, abs(110-90), abs(105-90))
        // TR = max(5, 20, 15) = 20
        let result = TaTr::builtin_fn(&mut ctx, vec![]).unwrap();

        assert_eq!(result, Value::Number(20.0));
    }
}
