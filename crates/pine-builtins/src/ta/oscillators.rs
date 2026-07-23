use super::moving_averages::{checked_length, smooth_step};
use pine_builtin_macro::BuiltinFunction;
use pine_interpreter::{Interpreter, PineOutput, RuntimeError, SeriesBuffer, Value};

/// ta.rsi(source, length) - Relative Strength Index
///
/// `100 - 100 / (1 + rs)`, where `rs` is Wilder-smoothed gains over Wilder-
/// smoothed losses. Both averages are carried across bars by the call site, so
/// this is the real recursive definition rather than an average of the last
/// `length` changes.
#[derive(BuiltinFunction)]
#[builtin(name = "ta.rsi", stateful)]
pub struct TaRsi {
    source: f64,
    length: f64,
    /// Last bar's source, to difference against. `None` on the first bar, when
    /// there is no change to measure yet.
    #[state]
    previous: Option<f64>,
    #[state]
    gains: SeriesBuffer<f64>,
    #[state]
    losses: SeriesBuffer<f64>,
    #[state]
    avg_gain: Option<f64>,
    #[state]
    avg_loss: Option<f64>,
}

impl TaRsi {
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

        let Some(previous) = self.previous.replace(self.source) else {
            // First bar: no previous value, so no change to measure yet.
            return Ok(Value::Na);
        };

        let change = self.source - previous;
        let gain = change.max(0.0);
        let loss = (-change).max(0.0);

        // Both sides advance together, so they fill on the same bar.
        let gain_seed = self.gains.observe(gain, length);
        let loss_seed = self.losses.observe(loss, length);
        let (Some(gain_seed), Some(loss_seed)) = (gain_seed, loss_seed) else {
            return Ok(Value::Na);
        };

        let alpha = 1.0 / length as f64;
        let avg_gain = smooth_step(self.avg_gain, gain, alpha, &gain_seed);
        let avg_loss = smooth_step(self.avg_loss, loss, alpha, &loss_seed);
        self.avg_gain = Some(avg_gain);
        self.avg_loss = Some(avg_loss);

        // Only-gains saturates at 100, only-losses at 0, and no movement at all
        // sits in the middle.
        if avg_loss == 0.0 {
            return Ok(Value::Number(if avg_gain == 0.0 { 50.0 } else { 100.0 }));
        }

        let rs = avg_gain / avg_loss;
        Ok(Value::Number(100.0 - 100.0 / (1.0 + rs)))
    }
}

/// ta.cci(source, length) - Commodity Channel Index
#[derive(BuiltinFunction)]
#[builtin(name = "ta.cci", stateful)]
pub struct TaCci {
    source: f64,
    length: f64,
    #[state]
    window: SeriesBuffer<f64>,
}

impl TaCci {
    fn execute<O: PineOutput>(
        &mut self,
        _ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let length = checked_length(self.length)?;

        let Some(values) = self.window.observe(self.source, length) else {
            return Ok(Value::Na);
        };

        let sma: f64 = values.iter().sum::<f64>() / length as f64;
        let mad: f64 = values.iter().map(|&v| (v - sma).abs()).sum::<f64>() / length as f64;

        if mad == 0.0 {
            return Ok(Value::Na);
        }

        Ok(Value::Number((values[0] - sma) / (0.015 * mad)))
    }
}

/// ta.mom(source, length) - Momentum: the change over `length` bars.
#[derive(BuiltinFunction)]
#[builtin(name = "ta.mom", stateful)]
pub struct TaMom {
    source: f64,
    length: f64,
    #[state]
    window: SeriesBuffer<f64>,
}

impl TaMom {
    fn execute<O: PineOutput>(
        &mut self,
        _ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let length = self.length as usize;

        // `length` bars back needs `length + 1` values in hand.
        let Some(values) = self.window.observe(self.source, length + 1) else {
            return Ok(Value::Na);
        };

        Ok(Value::Number(values[0] - values[length]))
    }
}

/// ta.roc(source, length) - Rate of Change, as a percentage.
#[derive(BuiltinFunction)]
#[builtin(name = "ta.roc", stateful)]
pub struct TaRoc {
    source: f64,
    length: f64,
    #[state]
    window: SeriesBuffer<f64>,
}

impl TaRoc {
    fn execute<O: PineOutput>(
        &mut self,
        _ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let length = self.length as usize;

        let Some(values) = self.window.observe(self.source, length + 1) else {
            return Ok(Value::Na);
        };

        let previous = values[length];
        if previous == 0.0 {
            return Ok(Value::Na);
        }

        Ok(Value::Number((values[0] - previous) / previous * 100.0))
    }
}

/// ta.cmo(source, length) - Chande Momentum Oscillator
#[derive(BuiltinFunction)]
#[builtin(name = "ta.cmo", stateful)]
pub struct TaCmo {
    source: f64,
    length: f64,
    #[state]
    window: SeriesBuffer<f64>,
}

impl TaCmo {
    fn execute<O: PineOutput>(
        &mut self,
        _ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let length = checked_length(self.length)?;

        // `length` changes need `length + 1` values in hand.
        let Some(values) = self.window.observe(self.source, length + 1) else {
            return Ok(Value::Na);
        };

        let mut gains = 0.0;
        let mut losses = 0.0;
        for pair in values.windows(2) {
            let change = pair[0] - pair[1];
            if change > 0.0 {
                gains += change;
            } else {
                losses -= change;
            }
        }

        let total = gains + losses;
        if total == 0.0 {
            return Ok(Value::Number(0.0));
        }

        Ok(Value::Number(100.0 * (gains - losses) / total))
    }
}

/// ta.stoch(source, high, low, length) - Stochastic: where `source` sits inside
/// the `[lowest(low), highest(high)]` range of the last `length` bars, as 0..100.
#[derive(BuiltinFunction)]
#[builtin(name = "ta.stoch", stateful)]
pub struct TaStoch {
    source: f64,
    high: f64,
    low: f64,
    length: f64,
    #[state]
    highs: SeriesBuffer<f64>,
    #[state]
    lows: SeriesBuffer<f64>,
}

impl TaStoch {
    fn execute<O: PineOutput>(
        &mut self,
        _ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let length = checked_length(self.length)?;

        // Both sides advance together, so they fill on the same bar.
        let highs = self.highs.observe(self.high, length);
        let lows = self.lows.observe(self.low, length);
        let (Some(highs), Some(lows)) = (highs, lows) else {
            return Ok(Value::Na);
        };

        let highest = highs.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let lowest = lows.iter().copied().fold(f64::INFINITY, f64::min);

        let range = highest - lowest;
        if range == 0.0 {
            return Ok(Value::Number(0.0));
        }

        Ok(Value::Number(100.0 * (self.source - lowest) / range))
    }
}

/// ta.mfi(source, length) - Money Flow Index.
///
/// Mirrors the reference implementation the spec gives:
///
/// ```text
/// upper = math.sum(volume * (ta.change(src) <= 0.0 ? 0.0 : src), length)
/// lower = math.sum(volume * (ta.change(src) >= 0.0 ? 0.0 : src), length)
/// mfi   = 100.0 - (100.0 / (1.0 + upper / lower))
/// ```
///
/// On the first bar `ta.change` is na, and an na ternary condition takes the
/// else branch — so that bar's flow counts towards *both* sums rather than
/// being skipped.
#[derive(BuiltinFunction)]
#[builtin(name = "ta.mfi", stateful)]
pub struct TaMfi {
    source: f64,
    length: f64,
    #[state]
    upper: SeriesBuffer<f64>,
    #[state]
    lower: SeriesBuffer<f64>,
    #[state]
    previous: Option<f64>,
}

impl TaMfi {
    fn execute<O: PineOutput>(
        &mut self,
        ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let length = checked_length(self.length)?;

        let volume = ctx
            .get_variable("volume")
            .ok_or_else(|| RuntimeError::UndefinedVariable("volume".to_string()))?
            .as_number()?;

        let flow = volume * self.source;
        // `None` is the first bar's na change, which fails both `<= 0.0` and
        // `>= 0.0` and so takes the else branch on each side.
        let change = self.previous.replace(self.source).map(|p| self.source - p);
        let upper = self.upper.observe(
            if change.is_some_and(|c| c <= 0.0) {
                0.0
            } else {
                flow
            },
            length,
        );
        let lower = self.lower.observe(
            if change.is_some_and(|c| c >= 0.0) {
                0.0
            } else {
                flow
            },
            length,
        );

        let (Some(upper), Some(lower)) = (upper, lower) else {
            return Ok(Value::Na);
        };

        let upper: f64 = upper.iter().sum();
        let lower: f64 = lower.iter().sum();

        // No down bars in the window means the index is pinned at its top.
        if lower == 0.0 {
            return Ok(Value::Number(100.0));
        }

        Ok(Value::Number(100.0 - 100.0 / (1.0 + upper / lower)))
    }
}

/// ta.linreg(source, length, offset) - Linear Regression
#[derive(BuiltinFunction)]
#[builtin(name = "ta.linreg", stateful)]
pub struct TaLinreg {
    source: f64,
    length: f64,
    #[arg(default = 0.0)]
    offset: f64,
    #[state]
    window: SeriesBuffer<f64>,
}

impl TaLinreg {
    fn execute<O: PineOutput>(
        &mut self,
        _ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let length = checked_length(self.length)?;

        let Some(values) = self.window.observe(self.source, length) else {
            return Ok(Value::Na);
        };

        let n = values.len() as f64;
        let mean_x = (values.len() - 1) as f64 / 2.0;
        let mean_y: f64 = values.iter().sum::<f64>() / n;

        let mut numerator = 0.0;
        let mut denominator = 0.0;
        for (i, &value) in values.iter().enumerate() {
            let x_dev = i as f64 - mean_x;
            numerator += x_dev * (value - mean_y);
            denominator += x_dev * x_dev;
        }

        if denominator == 0.0 {
            return Ok(Value::Number(mean_y));
        }

        let slope = numerator / denominator;
        let intercept = mean_y - slope * mean_x;

        Ok(Value::Number(intercept + slope * self.offset))
    }
}
