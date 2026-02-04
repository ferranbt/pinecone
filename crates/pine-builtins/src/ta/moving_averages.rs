use pine_builtin_macro::BuiltinFunction;
use pine_interpreter::{Interpreter, RuntimeError, Value};

/// ta.sma(source, length) - Simple Moving Average
#[derive(BuiltinFunction)]
#[builtin(name = "ta.sma")]
pub struct TaSma {
    source: Value,
    length: f64,
}

impl TaSma {
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

        let sum: f64 = values.iter().sum();
        let avg = sum / values.len() as f64;
        Ok(Value::Number(avg))
    }
}

/// ta.ema(source, length) - Exponential Moving Average
#[derive(BuiltinFunction)]
#[builtin(name = "ta.ema")]
pub struct TaEma {
    source: Value,
    length: f64,
}

impl TaEma {
    fn execute(&self, ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let length = self.length as usize;
        if length == 0 {
            return Err(RuntimeError::TypeError(
                "length must be greater than 0".to_string(),
            ));
        }

        // EMA calculation: EMA = (Close - EMA(prev)) * multiplier + EMA(prev)
        // where multiplier = 2 / (length + 1)
        let multiplier = 2.0 / (length as f64 + 1.0);

        if let Value::Series(series) = &self.source {
            let mut values = Vec::new();

            // Get current value
            if let Value::Number(n) = *series.current {
                values.push(n);
            } else {
                return Err(RuntimeError::TypeError(
                    "Series must contain numbers".to_string(),
                ));
            }

            // Get historical values
            if let Some(provider) = &ctx.historical_provider {
                for i in 1..length * 2 {
                    // Get more data for better EMA accuracy
                    if let Some(Value::Number(n)) = provider.get_historical(&series.id, i) {
                        values.push(n);
                    } else {
                        break;
                    }
                }
            }

            if values.is_empty() {
                return Ok(Value::Na);
            }

            // Start with SMA for first value
            let initial_sma: f64 = values.iter().take(length.min(values.len())).sum::<f64>()
                / length.min(values.len()) as f64;

            // Calculate EMA backwards from oldest to newest
            let mut ema = initial_sma;
            for &val in values
                .iter()
                .rev()
                .skip(length.min(values.len()))
                .take(values.len() - length.min(values.len()))
            {
                ema = (val - ema) * multiplier + ema;
            }

            // Apply final value (current)
            ema = (values[0] - ema) * multiplier + ema;

            Ok(Value::Number(ema))
        } else if let Value::Number(n) = self.source {
            Ok(Value::Number(n))
        } else {
            Err(RuntimeError::TypeError(
                "source must be a number or series".to_string(),
            ))
        }
    }
}

/// ta.rma(source, length) - Rolling Moving Average (Wilder's Smoothing)
#[derive(BuiltinFunction)]
#[builtin(name = "ta.rma")]
pub struct TaRma {
    source: Value,
    length: f64,
}

impl TaRma {
    fn execute(&self, ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let length = self.length as usize;
        if length == 0 {
            return Err(RuntimeError::TypeError(
                "length must be greater than 0".to_string(),
            ));
        }

        // RMA calculation: RMA = (prev_RMA * (length - 1) + current) / length
        // This is equivalent to EMA with alpha = 1/length

        if let Value::Series(series) = &self.source {
            let mut values = Vec::new();

            // Get current value
            if let Value::Number(n) = *series.current {
                values.push(n);
            } else {
                return Err(RuntimeError::TypeError(
                    "Series must contain numbers".to_string(),
                ));
            }

            // Get historical values
            if let Some(provider) = &ctx.historical_provider {
                for i in 1..length * 2 {
                    if let Some(Value::Number(n)) = provider.get_historical(&series.id, i) {
                        values.push(n);
                    } else {
                        break;
                    }
                }
            }

            if values.is_empty() {
                return Ok(Value::Na);
            }

            // Start with SMA
            let initial_sma: f64 = values.iter().take(length.min(values.len())).sum::<f64>()
                / length.min(values.len()) as f64;

            // Calculate RMA
            let mut rma = initial_sma;
            let alpha = 1.0 / length as f64;

            for &val in values
                .iter()
                .rev()
                .skip(length.min(values.len()))
                .take(values.len() - length.min(values.len()))
            {
                rma = alpha * val + (1.0 - alpha) * rma;
            }

            // Apply current value
            rma = alpha * values[0] + (1.0 - alpha) * rma;

            Ok(Value::Number(rma))
        } else if let Value::Number(n) = self.source {
            Ok(Value::Number(n))
        } else {
            Err(RuntimeError::TypeError(
                "source must be a number or series".to_string(),
            ))
        }
    }
}

/// ta.wma(source, length) - Weighted Moving Average
#[derive(BuiltinFunction)]
#[builtin(name = "ta.wma")]
pub struct TaWma {
    source: Value,
    length: f64,
}

impl TaWma {
    fn execute(&self, ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let length = self.length as usize;
        if length == 0 {
            return Err(RuntimeError::TypeError(
                "length must be greater than 0".to_string(),
            ));
        }

        // WMA calculation: sum(price[i] * (length - i)) / sum(length - i)
        // Most recent data has highest weight

        if let Value::Series(series) = &self.source {
            let mut values = Vec::new();

            // Get current value
            if let Value::Number(n) = *series.current {
                values.push(n);
            } else {
                return Err(RuntimeError::TypeError(
                    "Series must contain numbers".to_string(),
                ));
            }

            // Get historical values
            if let Some(provider) = &ctx.historical_provider {
                for i in 1..length {
                    if let Some(Value::Number(n)) = provider.get_historical(&series.id, i) {
                        values.push(n);
                    } else {
                        break;
                    }
                }
            }

            if values.is_empty() {
                return Ok(Value::Na);
            }

            let actual_length = values.len();
            let mut weighted_sum = 0.0;
            let mut weight_sum = 0.0;

            for (i, &val) in values.iter().enumerate() {
                let weight = (actual_length - i) as f64;
                weighted_sum += val * weight;
                weight_sum += weight;
            }

            Ok(Value::Number(weighted_sum / weight_sum))
        } else if let Value::Number(n) = self.source {
            Ok(Value::Number(n))
        } else {
            Err(RuntimeError::TypeError(
                "source must be a number or series".to_string(),
            ))
        }
    }
}

/// ta.vwma(source, length) - Volume Weighted Moving Average
/// Note: Requires volume series to be available in context
#[derive(BuiltinFunction)]
#[builtin(name = "ta.vwma")]
pub struct TaVwma {
    source: Value,
    length: f64,
}

impl TaVwma {
    fn execute(&self, ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let length = self.length as usize;
        if length == 0 {
            return Err(RuntimeError::TypeError(
                "length must be greater than 0".to_string(),
            ));
        }

        // Get price values
        let price_values = ctx.get_series_values(&self.source, length)?;

        if price_values.is_empty() {
            return Ok(Value::Na);
        }

        // Get volume series from context
        let volume = ctx
            .get_variable("volume")
            .ok_or_else(|| RuntimeError::UndefinedVariable("volume".to_string()))?;

        let volume_values = ctx.get_series_values(volume, length)?;

        if volume_values.len() != price_values.len() {
            return Ok(Value::Na);
        }

        // VWMA = sum(price * volume) / sum(volume)
        let mut weighted_sum = 0.0;
        let mut volume_sum = 0.0;

        for i in 0..price_values.len() {
            weighted_sum += price_values[i] * volume_values[i];
            volume_sum += volume_values[i];
        }

        if volume_sum == 0.0 {
            return Ok(Value::Na);
        }

        Ok(Value::Number(weighted_sum / volume_sum))
    }
}

/// ta.hma(source, length) - Hull Moving Average
#[derive(BuiltinFunction)]
#[builtin(name = "ta.hma")]
pub struct TaHma {
    source: Value,
    length: f64,
}

impl TaHma {
    fn execute(&self, ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let length = self.length as usize;
        if length == 0 {
            return Err(RuntimeError::TypeError(
                "length must be greater than 0".to_string(),
            ));
        }

        // HMA = WMA(2 * WMA(n/2) - WMA(n), sqrt(n))
        // Simplified implementation: just use WMA of source
        let half_length = (length as f64 / 2.0).floor() as usize;
        let sqrt_length = (length as f64).sqrt().floor() as usize;

        // Get values for calculations
        let values = ctx.get_series_values(&self.source, length)?;

        if values.is_empty() || values.len() < sqrt_length {
            return Ok(Value::Na);
        }

        // Calculate WMA(n/2)
        let wma_half = Self::calc_wma(&values[..half_length.min(values.len())]);

        // Calculate WMA(n)
        let wma_full = Self::calc_wma(&values);

        // 2 * WMA(n/2) - WMA(n)
        let raw_hma = 2.0 * wma_half - wma_full;

        // For simplicity, return the raw value instead of recalculating WMA
        // Full implementation would need state across bars
        Ok(Value::Number(raw_hma))
    }

    fn calc_wma(values: &[f64]) -> f64 {
        if values.is_empty() {
            return 0.0;
        }

        let len = values.len();
        let mut weighted_sum = 0.0;
        let mut weight_sum = 0.0;

        for (i, &val) in values.iter().enumerate() {
            let weight = (len - i) as f64;
            weighted_sum += val * weight;
            weight_sum += weight;
        }

        if weight_sum == 0.0 {
            0.0
        } else {
            weighted_sum / weight_sum
        }
    }
}

/// ta.swma(source) - Symmetrically Weighted Moving Average (4-bar)
#[derive(BuiltinFunction)]
#[builtin(name = "ta.swma")]
pub struct TaSwma {
    source: Value,
}

impl TaSwma {
    fn execute(&self, ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        // SWMA uses fixed 4-bar window with weights [1, 2, 2, 1]
        let values = ctx.get_series_values(&self.source, 4)?;

        if values.len() < 4 {
            return Ok(Value::Na);
        }

        // Weights: 1, 2, 2, 1 (symmetrical)
        let swma = (values[0] * 1.0 + values[1] * 2.0 + values[2] * 2.0 + values[3] * 1.0) / 6.0;

        Ok(Value::Number(swma))
    }
}
