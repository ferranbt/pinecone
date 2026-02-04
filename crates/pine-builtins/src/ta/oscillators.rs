use pine_builtin_macro::BuiltinFunction;
use pine_interpreter::{Interpreter, RuntimeError, Value};

/// ta.rsi(source, length) - Relative Strength Index
#[derive(BuiltinFunction)]
#[builtin(name = "ta.rsi")]
pub struct TaRsi {
    source: Value,
    length: f64,
}

impl TaRsi {
    fn execute(&self, ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let length = self.length as usize;
        if length == 0 {
            return Err(RuntimeError::TypeError(
                "length must be greater than 0".to_string(),
            ));
        }

        // Get enough values to calculate changes
        let values = ctx.get_series_values(&self.source, length + 1)?;

        if values.len() < 2 {
            return Ok(Value::Na);
        }

        // Calculate gains and losses
        let mut gains = Vec::new();
        let mut losses = Vec::new();

        for i in 0..values.len() - 1 {
            let change = values[i] - values[i + 1];
            if change > 0.0 {
                gains.push(change);
                losses.push(0.0);
            } else {
                gains.push(0.0);
                losses.push(-change);
            }
        }

        if gains.is_empty() {
            return Ok(Value::Na);
        }

        // Calculate RMA of gains and losses
        let avg_gain: f64 = gains.iter().take(length.min(gains.len())).sum::<f64>()
            / length.min(gains.len()) as f64;
        let avg_loss: f64 = losses.iter().take(length.min(losses.len())).sum::<f64>()
            / length.min(losses.len()) as f64;

        if avg_loss == 0.0 {
            return Ok(Value::Number(100.0));
        }

        let rs = avg_gain / avg_loss;
        let rsi = 100.0 - (100.0 / (1.0 + rs));

        Ok(Value::Number(rsi))
    }
}

/// ta.cci(source, length) - Commodity Channel Index
#[derive(BuiltinFunction)]
#[builtin(name = "ta.cci")]
pub struct TaCci {
    source: Value,
    length: f64,
}

impl TaCci {
    fn execute(&self, ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let length = self.length as usize;
        if length == 0 {
            return Err(RuntimeError::TypeError(
                "length must be greater than 0".to_string(),
            ));
        }

        // CCI = (Typical Price - SMA) / (0.015 * Mean Deviation)
        // For simplicity, we use source directly instead of typical price
        let values = ctx.get_series_values(&self.source, length)?;

        if values.is_empty() {
            return Ok(Value::Na);
        }

        // Calculate SMA
        let sma: f64 = values.iter().sum::<f64>() / values.len() as f64;

        // Calculate mean absolute deviation
        let mad: f64 = values.iter().map(|&v| (v - sma).abs()).sum::<f64>() / values.len() as f64;

        if mad == 0.0 {
            return Ok(Value::Na);
        }

        let cci = (values[0] - sma) / (0.015 * mad);
        Ok(Value::Number(cci))
    }
}

/// ta.mom(source, length) - Momentum
#[derive(BuiltinFunction)]
#[builtin(name = "ta.mom")]
pub struct TaMom {
    source: Value,
    length: f64,
}

impl TaMom {
    fn execute(&self, ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let length = self.length as usize;

        // Momentum is just current - value N bars ago
        let values = ctx.get_series_values(&self.source, length + 1)?;

        if values.len() <= length {
            return Ok(Value::Na);
        }

        let momentum = values[0] - values[length];
        Ok(Value::Number(momentum))
    }
}

/// ta.roc(source, length) - Rate of Change
#[derive(BuiltinFunction)]
#[builtin(name = "ta.roc")]
pub struct TaRoc {
    source: Value,
    length: f64,
}

impl TaRoc {
    fn execute(&self, ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let length = self.length as usize;

        // ROC = ((current - previous) / previous) * 100
        let values = ctx.get_series_values(&self.source, length + 1)?;

        if values.len() <= length {
            return Ok(Value::Na);
        }

        let current = values[0];
        let previous = values[length];

        if previous == 0.0 {
            return Ok(Value::Na);
        }

        let roc = ((current - previous) / previous) * 100.0;
        Ok(Value::Number(roc))
    }
}

/// ta.cmo(source, length) - Chande Momentum Oscillator
#[derive(BuiltinFunction)]
#[builtin(name = "ta.cmo")]
pub struct TaCmo {
    source: Value,
    length: f64,
}

impl TaCmo {
    fn execute(&self, ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let length = self.length as usize;
        if length == 0 {
            return Err(RuntimeError::TypeError(
                "length must be greater than 0".to_string(),
            ));
        }

        // CMO = 100 * (sum of gains - sum of losses) / (sum of gains + sum of losses)
        let values = ctx.get_series_values(&self.source, length + 1)?;

        if values.len() < 2 {
            return Ok(Value::Na);
        }

        let mut sum_gains = 0.0;
        let mut sum_losses = 0.0;

        for i in 0..values.len() - 1 {
            let change = values[i] - values[i + 1];
            if change > 0.0 {
                sum_gains += change;
            } else {
                sum_losses += -change;
            }
        }

        let total = sum_gains + sum_losses;
        if total == 0.0 {
            return Ok(Value::Number(0.0));
        }

        let cmo = 100.0 * (sum_gains - sum_losses) / total;
        Ok(Value::Number(cmo))
    }
}

/// ta.linreg(source, length, offset) - Linear Regression
#[derive(BuiltinFunction)]
#[builtin(name = "ta.linreg")]
pub struct TaLinreg {
    source: Value,
    length: f64,
    #[arg(default = 0.0)]
    offset: f64,
}

impl TaLinreg {
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

        let n = values.len() as f64;

        // Calculate means
        let mean_x = (values.len() - 1) as f64 / 2.0;
        let mean_y: f64 = values.iter().sum::<f64>() / n;

        // Calculate slope (beta)
        let mut numerator = 0.0;
        let mut denominator = 0.0;

        for (i, &val) in values.iter().enumerate() {
            let x_dev = i as f64 - mean_x;
            numerator += x_dev * (val - mean_y);
            denominator += x_dev * x_dev;
        }

        if denominator == 0.0 {
            return Ok(Value::Number(mean_y));
        }

        let slope = numerator / denominator;
        let intercept = mean_y - slope * mean_x;

        // Calculate value at offset (0 = current bar)
        let x_pos = self.offset;
        let linreg_val = intercept + slope * x_pos;

        Ok(Value::Number(linreg_val))
    }
}
