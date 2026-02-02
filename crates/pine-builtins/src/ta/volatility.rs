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
        let high = ctx.get_variable("high")
            .ok_or_else(|| RuntimeError::UndefinedVariable("high".to_string()))?;
        let low = ctx.get_variable("low")
            .ok_or_else(|| RuntimeError::UndefinedVariable("low".to_string()))?;
        let close = ctx.get_variable("close")
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
            return Err(RuntimeError::TypeError("high must be a number or series".to_string()));
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
            return Err(RuntimeError::TypeError("low must be a number or series".to_string()));
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
            return Err(RuntimeError::TypeError("length must be greater than 0".to_string()));
        }

        // Get high, low, close series from context
        let high = ctx.get_variable("high")
            .ok_or_else(|| RuntimeError::UndefinedVariable("high".to_string()))?;
        let low = ctx.get_variable("low")
            .ok_or_else(|| RuntimeError::UndefinedVariable("low".to_string()))?;
        let close = ctx.get_variable("close")
            .ok_or_else(|| RuntimeError::UndefinedVariable("close".to_string()))?;

        // We need to calculate TR for each bar and then RMA of those TRs
        let mut tr_values = Vec::new();

        // Calculate current TR
        let current_tr = {
            let high_val = if let Value::Series(s) = high {
                if let Value::Number(n) = *s.current { n } else {
                    return Err(RuntimeError::TypeError("high must contain numbers".to_string()));
                }
            } else if let Value::Number(n) = high {
                *n
            } else {
                return Err(RuntimeError::TypeError("high must be a number or series".to_string()));
            };

            let low_val = if let Value::Series(s) = low {
                if let Value::Number(n) = *s.current { n } else {
                    return Err(RuntimeError::TypeError("low must contain numbers".to_string()));
                }
            } else if let Value::Number(n) = low {
                *n
            } else {
                return Err(RuntimeError::TypeError("low must be a number or series".to_string()));
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
        if let (Value::Series(high_s), Value::Series(low_s), Value::Series(close_s)) = (high, low, close) {
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

                    let pc = if let Some(Value::Number(n)) = provider.get_historical(&close_s.id, i + 1) {
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
        let initial_sma: f64 = tr_values.iter().take(length.min(tr_values.len())).sum::<f64>() / length.min(tr_values.len()) as f64;
        let mut rma = initial_sma;
        let alpha = 1.0 / length as f64;

        for &tr in tr_values.iter().rev().skip(length.min(tr_values.len())).take(tr_values.len() - length.min(tr_values.len())) {
            rma = alpha * tr + (1.0 - alpha) * rma;
        }

        // Apply current TR
        rma = alpha * tr_values[0] + (1.0 - alpha) * rma;

        Ok(Value::Number(rma))
    }
}
