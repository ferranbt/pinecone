use super::moving_averages::{checked_length, smooth_step};
use pine_builtin_macro::BuiltinFunction;
use pine_core::{PineOutput, SeriesBuffer};
use pine_interpreter::{Interpreter, RuntimeError, Value};
use std::cell::RefCell;
use std::rc::Rc;

/// This bar's true range: `max(high - low, |high - close[1]|, |low - close[1]|)`.
///
/// `previous_close` is `None` on the first bar, where Pine falls back to
/// `high - low` unless `handle_na` asks for na instead.
fn true_range(high: f64, low: f64, previous_close: Option<f64>) -> Option<f64> {
    match previous_close {
        Some(close) => Some(
            (high - low)
                .max((high - close).abs())
                .max((low - close).abs()),
        ),
        None => Some(high - low),
    }
}

/// Reads this bar's high, low and close, which the range builtins all need.
fn hlc<O: PineOutput>(ctx: &Interpreter<O>) -> Result<(f64, f64, f64), RuntimeError> {
    let read = |name: &str| -> Result<f64, RuntimeError> {
        ctx.get_variable(name)
            .ok_or_else(|| RuntimeError::UndefinedVariable(name.to_string()))?
            .as_number()
    };
    Ok((read("high")?, read("low")?, read("close")?))
}

/// ta.tr(handle_na) - True Range
#[derive(BuiltinFunction)]
#[builtin(name = "ta.tr", stateful)]
pub struct TaTr {
    #[arg(default = false)]
    handle_na: bool,
    /// Previous bar's close, which the range is measured against.
    #[state]
    previous_close: Option<f64>,
}

impl TaTr {
    fn execute<O: PineOutput>(
        &mut self,
        ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let (high, low, close) = hlc(ctx)?;
        let previous_close = self.previous_close.replace(close);

        // With no previous close, `handle_na = true` asks for na rather than
        // falling back to the bar's own range.
        if previous_close.is_none() && self.handle_na {
            return Ok(Value::Na);
        }

        match true_range(high, low, previous_close) {
            Some(tr) => Ok(Value::Number(tr)),
            None => Ok(Value::Na),
        }
    }
}

/// ta.atr(length) - Average True Range: Wilder-smoothed [`TaTr`].
#[derive(BuiltinFunction)]
#[builtin(name = "ta.atr", stateful)]
pub struct TaAtr {
    length: f64,
    #[state]
    previous_close: Option<f64>,
    #[state]
    window: SeriesBuffer<f64>,
    #[state]
    previous: Option<f64>,
}

impl TaAtr {
    fn execute<O: PineOutput>(
        &mut self,
        ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let length = checked_length(self.length)?;

        let (high, low, close) = hlc(ctx)?;
        let previous_close = self.previous_close.replace(close);
        let Some(tr) = true_range(high, low, previous_close) else {
            return Ok(Value::Na);
        };

        let Some(seed) = self.window.observe(tr, length) else {
            return Ok(Value::Na);
        };

        let atr = smooth_step(self.previous, tr, 1.0 / length as f64, &seed);
        self.previous = Some(atr);

        Ok(Value::Number(atr))
    }
}

/// ta.bb(series, length, mult) - Bollinger Bands, as `[middle, upper, lower]`.
#[derive(BuiltinFunction)]
#[builtin(name = "ta.bb", stateful)]
pub struct TaBb {
    series: f64,
    length: f64,
    mult: f64,
    #[state]
    window: SeriesBuffer<f64>,
}

impl TaBb {
    fn execute<O: PineOutput>(
        &mut self,
        _ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let length = checked_length(self.length)?;

        let Some(values) = self.window.observe(self.series, length) else {
            return Ok(bands(Value::Na, Value::Na, Value::Na));
        };

        let basis: f64 = values.iter().sum::<f64>() / length as f64;
        let variance: f64 = values
            .iter()
            .map(|value| (value - basis).powi(2))
            .sum::<f64>()
            / length as f64;

        let deviation = self.mult * variance.sqrt();
        Ok(bands(
            Value::Number(basis),
            Value::Number(basis + deviation),
            Value::Number(basis - deviation),
        ))
    }
}

/// The `[middle, upper, lower]` tuple `ta.bb` returns.
fn bands<O: PineOutput>(middle: Value<O>, upper: Value<O>, lower: Value<O>) -> Value<O> {
    Value::Array(Rc::new(RefCell::new(vec![middle, upper, lower])))
}

/// ta.sar(start, inc, max) - Parabolic SAR.
///
/// A direct port of TradingView's reference implementation: the first bar has no
/// prior close to set a direction, the second seeds the trend from `close` vs
/// `close[1]`, and each bar after advances the stop, flipping when price crosses
/// it and clamping the stop to the last two highs/lows.
#[derive(BuiltinFunction)]
#[builtin(name = "ta.sar", stateful)]
pub struct TaSar {
    #[arg(default = 0.02)]
    start: f64,
    #[arg(default = 0.02)]
    inc: f64,
    #[arg(default = 0.2)]
    max: f64,
    #[state]
    initialized: bool,
    #[state]
    result: f64,
    #[state]
    max_min: f64,
    #[state]
    acceleration: f64,
    /// True while price is above the stop (an up-trend).
    #[state]
    is_below: bool,
    #[state]
    prev_close: Option<f64>,
    /// Previous two bars' highs/lows (`high[1]`/`high[2]`), shifted each bar.
    #[state]
    high1: Option<f64>,
    #[state]
    high2: Option<f64>,
    #[state]
    low1: Option<f64>,
    #[state]
    low2: Option<f64>,
}

impl TaSar {
    fn execute<O: PineOutput>(
        &mut self,
        ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let (high, low, close) = hlc(ctx)?;

        let out = if let Some(prev_close) = self.prev_close {
            if !self.initialized {
                // Second bar: seed the trend from this close vs the last.
                self.initialized = true;
                if close > prev_close {
                    self.is_below = true;
                    self.max_min = high;
                    self.result = self.low1.unwrap_or(low);
                } else {
                    self.is_below = false;
                    self.max_min = low;
                    self.result = self.high1.unwrap_or(high);
                }
                self.acceleration = self.start;
                self.result
            } else {
                // Subsequent bars: advance the stop and flip on a cross.
                self.result += self.acceleration * (self.max_min - self.result);
                let mut flipped = false;
                if self.is_below {
                    if self.result > low {
                        flipped = true;
                        self.is_below = false;
                        self.result = self.max_min;
                        self.max_min = low;
                        self.acceleration = self.start;
                    }
                } else if self.result < high {
                    flipped = true;
                    self.is_below = true;
                    self.result = self.max_min;
                    self.max_min = high;
                    self.acceleration = self.start;
                }

                if !flipped {
                    if self.is_below {
                        if high > self.max_min {
                            self.max_min = high;
                            self.acceleration = (self.acceleration + self.inc).min(self.max);
                        }
                    } else if low < self.max_min {
                        self.max_min = low;
                        self.acceleration = (self.acceleration + self.inc).min(self.max);
                    }
                }

                // The stop can't enter the last two bars' range.
                if self.is_below {
                    if let Some(l1) = self.low1 {
                        self.result = self.result.min(l1);
                    }
                    if let Some(l2) = self.low2 {
                        self.result = self.result.min(l2);
                    }
                } else {
                    if let Some(h1) = self.high1 {
                        self.result = self.result.max(h1);
                    }
                    if let Some(h2) = self.high2 {
                        self.result = self.result.max(h2);
                    }
                }
                self.result
            }
        } else {
            // First bar: no prior close, so no direction yet.
            f64::NAN
        };

        // Shift the high/low history and record this close for the next bar.
        self.low2 = self.low1;
        self.low1 = Some(low);
        self.high2 = self.high1;
        self.high1 = Some(high);
        self.prev_close = Some(close);

        if out.is_nan() {
            Ok(Value::Na)
        } else {
            Ok(Value::Number(out))
        }
    }
}
