use super::moving_averages::{checked_length, ema_step, smooth_step, wilder_step};
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

/// ta.wpr(length) - Williams %R: `-100 * (highest_high - close) / (highest_high
/// - lowest_low)` over `length` bars (`0` when that range is `0`).
#[derive(BuiltinFunction)]
#[builtin(name = "ta.wpr", stateful)]
pub struct TaWpr {
    length: f64,
    #[state]
    highs: SeriesBuffer<f64>,
    #[state]
    lows: SeriesBuffer<f64>,
}

impl TaWpr {
    fn execute<O: PineOutput>(
        &mut self,
        ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let length = checked_length(self.length)?;
        let (high, low, close) = hlc(ctx)?;
        let highs = self.highs.observe(high, length);
        let Some(lows) = self.lows.observe(low, length) else {
            return Ok(Value::Na);
        };
        let highs = highs.expect("high and low windows fill together");
        let highest = highs.iter().copied().fold(f64::MIN, f64::max);
        let lowest = lows.iter().copied().fold(f64::MAX, f64::min);
        let range = highest - lowest;
        if range == 0.0 {
            return Ok(Value::Number(0.0));
        }
        Ok(Value::Number(-100.0 * (highest - close) / range))
    }
}

/// The `[a, b]` pair returned by `ta.supertrend`.
fn pair<O: PineOutput>(a: Value<O>, b: Value<O>) -> Value<O> {
    Value::Array(Rc::new(RefCell::new(vec![a, b])))
}

/// ta.dmi(diLength, adxSmoothing) - Directional Movement Index, as
/// `[di_plus, di_minus, adx]`.
///
/// Directional movement and true range are Wilder-smoothed over `diLength` into
/// the `+DI`/`-DI` lines; the ADX is the Wilder-smoothed directional index over
/// `adxSmoothing` bars.
#[derive(BuiltinFunction)]
#[builtin(name = "ta.dmi", stateful)]
pub struct TaDmi {
    di_length: f64,
    adx_smoothing: f64,
    #[state]
    previous_high: Option<f64>,
    #[state]
    previous_low: Option<f64>,
    #[state]
    previous_close: Option<f64>,
    #[state]
    tr_win: SeriesBuffer<f64>,
    #[state]
    tr_prev: Option<f64>,
    #[state]
    plus_win: SeriesBuffer<f64>,
    #[state]
    plus_prev: Option<f64>,
    #[state]
    minus_win: SeriesBuffer<f64>,
    #[state]
    minus_prev: Option<f64>,
    #[state]
    dx_win: SeriesBuffer<f64>,
    #[state]
    dx_prev: Option<f64>,
}

impl TaDmi {
    fn execute<O: PineOutput>(
        &mut self,
        ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let (di_length, adx_smoothing) = (
            checked_length(self.di_length)?,
            checked_length(self.adx_smoothing)?,
        );
        let na = || pair3(Value::Na, Value::Na, Value::Na);
        let (high, low, close) = hlc(ctx)?;
        let (Some(ph), Some(pl), Some(pc)) =
            (self.previous_high, self.previous_low, self.previous_close)
        else {
            // No prior bar: seed the previous values and emit na.
            self.previous_high = Some(high);
            self.previous_low = Some(low);
            self.previous_close = Some(close);
            return Ok(na());
        };
        let tr = (high - low).max((high - pc).abs()).max((low - pc).abs());
        let up_move = high - ph;
        let down_move = pl - low;
        let plus_dm = if up_move > down_move && up_move > 0.0 {
            up_move
        } else {
            0.0
        };
        let minus_dm = if down_move > up_move && down_move > 0.0 {
            down_move
        } else {
            0.0
        };
        self.previous_high = Some(high);
        self.previous_low = Some(low);
        self.previous_close = Some(close);

        let smoothed_tr = wilder_step(&mut self.tr_win, &mut self.tr_prev, tr, di_length);
        let smoothed_plus =
            wilder_step(&mut self.plus_win, &mut self.plus_prev, plus_dm, di_length);
        let smoothed_minus = wilder_step(
            &mut self.minus_win,
            &mut self.minus_prev,
            minus_dm,
            di_length,
        );
        let (Some(str_), Some(sp), Some(sm)) = (smoothed_tr, smoothed_plus, smoothed_minus) else {
            return Ok(na());
        };
        let di_plus = if str_ == 0.0 { 0.0 } else { 100.0 * sp / str_ };
        let di_minus = if str_ == 0.0 { 0.0 } else { 100.0 * sm / str_ };
        let sum = di_plus + di_minus;
        let dx = if sum == 0.0 {
            0.0
        } else {
            100.0 * (di_plus - di_minus).abs() / sum
        };
        let adx = wilder_step(&mut self.dx_win, &mut self.dx_prev, dx, adx_smoothing);
        Ok(pair3(
            Value::Number(di_plus),
            Value::Number(di_minus),
            adx.map_or(Value::Na, Value::Number),
        ))
    }
}

/// ta.supertrend(factor, atr_period) - Supertrend, as `[supertrend, direction]`
/// (direction `-1` in an uptrend, `1` in a downtrend).
#[derive(BuiltinFunction)]
#[builtin(name = "ta.supertrend", stateful)]
pub struct TaSupertrend {
    factor: f64,
    atr_period: f64,
    #[state]
    previous_close: Option<f64>,
    #[state]
    tr_win: SeriesBuffer<f64>,
    #[state]
    tr_prev: Option<f64>,
    #[state]
    previous_upper: Option<f64>,
    #[state]
    previous_lower: Option<f64>,
    #[state]
    previous_supertrend: Option<f64>,
}

impl TaSupertrend {
    fn execute<O: PineOutput>(
        &mut self,
        ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let atr_period = checked_length(self.atr_period)?;
        let (high, low, close) = hlc(ctx)?;
        let previous_close = self.previous_close.replace(close);
        // True range falls back to `high - low` on the first bar.
        let tr = match previous_close {
            Some(pc) => (high - low).max((high - pc).abs()).max((low - pc).abs()),
            None => high - low,
        };
        let Some(atr) = wilder_step(&mut self.tr_win, &mut self.tr_prev, tr, atr_period) else {
            return Ok(pair(Value::Na, Value::Na));
        };
        let hl2 = (high + low) / 2.0;
        let upper_basic = hl2 + self.factor * atr;
        let lower_basic = hl2 - self.factor * atr;
        // Carry the previous band unless price closed through it.
        let lower = match self.previous_lower {
            Some(prev) if lower_basic <= prev && previous_close.is_some_and(|c| c >= prev) => prev,
            _ => lower_basic,
        };
        let upper = match self.previous_upper {
            Some(prev) if upper_basic >= prev && previous_close.is_some_and(|c| c <= prev) => prev,
            _ => upper_basic,
        };
        let direction = if self.previous_supertrend.is_none() {
            1.0
        } else if self.previous_supertrend == self.previous_upper {
            if close > upper {
                -1.0
            } else {
                1.0
            }
        } else if close < lower {
            1.0
        } else {
            -1.0
        };
        let supertrend = if direction == -1.0 { lower } else { upper };
        self.previous_lower = Some(lower);
        self.previous_upper = Some(upper);
        self.previous_supertrend = Some(supertrend);
        Ok(pair(Value::Number(supertrend), Value::Number(direction)))
    }
}

/// The `[a, b, c]` triple returned by `ta.dmi`.
fn pair3<O: PineOutput>(a: Value<O>, b: Value<O>, c: Value<O>) -> Value<O> {
    Value::Array(Rc::new(RefCell::new(vec![a, b, c])))
}

/// ta.macd(source, fast, slow, signal) - `[macd, signal, histogram]`.
///
/// `macd = ema(source, fast) - ema(source, slow)`; the signal line is the EMA of
/// the macd line over `signal` bars, which only starts once the macd line is
/// non-na; the histogram is `macd - signal`.
#[derive(BuiltinFunction)]
#[builtin(name = "ta.macd", stateful)]
pub struct TaMacd {
    source: f64,
    fast: f64,
    slow: f64,
    signal: f64,
    #[state]
    fast_win: SeriesBuffer<f64>,
    #[state]
    fast_prev: Option<f64>,
    #[state]
    slow_win: SeriesBuffer<f64>,
    #[state]
    slow_prev: Option<f64>,
    #[state]
    sig_win: SeriesBuffer<f64>,
    #[state]
    sig_prev: Option<f64>,
}

impl TaMacd {
    fn execute<O: PineOutput>(
        &mut self,
        _ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let (fast, slow, signal) = (
            checked_length(self.fast)?,
            checked_length(self.slow)?,
            checked_length(self.signal)?,
        );
        let f = ema_step(&mut self.fast_win, &mut self.fast_prev, self.source, fast);
        let d = ema_step(&mut self.slow_win, &mut self.slow_prev, self.source, slow);
        let macd = match (f, d) {
            (Some(f), Some(d)) => Some(f - d),
            _ => None,
        };
        let sig = match macd {
            Some(m) => ema_step(&mut self.sig_win, &mut self.sig_prev, m, signal),
            None => None,
        };
        let hist = match (macd, sig) {
            (Some(m), Some(s)) => Some(m - s),
            _ => None,
        };
        let cell = |v: Option<f64>| v.map_or(Value::Na, Value::Number);
        Ok(bands(cell(macd), cell(sig), cell(hist)))
    }
}

/// ta.bbw(series, length, mult) - Bollinger Bands Width: `(upper - lower) / basis`.
///
/// The bands are `sma(series, length) ± mult * stdev(series, length)`, so the
/// width reduces to `2 * mult * stdev / basis` (`0` when the basis is `0`).
#[derive(BuiltinFunction)]
#[builtin(name = "ta.bbw", stateful)]
pub struct TaBbw {
    series: f64,
    length: f64,
    mult: f64,
    #[state]
    window: SeriesBuffer<f64>,
}

impl TaBbw {
    fn execute<O: PineOutput>(
        &mut self,
        _ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let length = checked_length(self.length)?;
        let Some(values) = self.window.observe(self.series, length) else {
            return Ok(Value::Na);
        };
        let basis = values.iter().sum::<f64>() / length as f64;
        if basis == 0.0 {
            return Ok(Value::Number(0.0));
        }
        let variance = values.iter().map(|v| (v - basis).powi(2)).sum::<f64>() / length as f64;
        Ok(Value::Number(2.0 * self.mult * variance.sqrt() / basis))
    }
}

/// ta.kc(series, length, mult, use_true_range) - Keltner Channels, as
/// `[middle, upper, lower]`.
///
/// The basis is `ema(series, length)`; the band offset is `mult` times the EMA
/// of the range, which is the true range when `use_true_range` (default) and
/// `high - low` otherwise.
#[derive(BuiltinFunction)]
#[builtin(name = "ta.kc", stateful)]
pub struct TaKc {
    series: f64,
    length: f64,
    mult: f64,
    #[arg(default = true)]
    use_true_range: bool,
    #[state]
    basis_win: SeriesBuffer<f64>,
    #[state]
    basis_prev: Option<f64>,
    #[state]
    range_win: SeriesBuffer<f64>,
    #[state]
    range_prev: Option<f64>,
    #[state]
    previous_close: Option<f64>,
}

impl TaKc {
    fn execute<O: PineOutput>(
        &mut self,
        ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let length = checked_length(self.length)?;
        let (high, low, close) = hlc(ctx)?;
        let previous_close = self.previous_close.replace(close);
        let range = if self.use_true_range {
            match previous_close {
                Some(pc) => (high - low).max((high - pc).abs()).max((low - pc).abs()),
                None => f64::NAN,
            }
        } else {
            high - low
        };
        let basis = ema_step(
            &mut self.basis_win,
            &mut self.basis_prev,
            self.series,
            length,
        );
        let range_ma = ema_step(&mut self.range_win, &mut self.range_prev, range, length);
        match (basis, range_ma) {
            (Some(basis), Some(range_ma)) => Ok(bands(
                Value::Number(basis),
                Value::Number(basis + self.mult * range_ma),
                Value::Number(basis - self.mult * range_ma),
            )),
            _ => Ok(bands(Value::Na, Value::Na, Value::Na)),
        }
    }
}

/// ta.kcw(series, length, mult, use_true_range) - Keltner Channels Width:
/// `(upper - lower) / basis`, i.e. `2 * mult * range_ma / basis`.
#[derive(BuiltinFunction)]
#[builtin(name = "ta.kcw", stateful)]
pub struct TaKcw {
    series: f64,
    length: f64,
    mult: f64,
    #[arg(default = true)]
    use_true_range: bool,
    #[state]
    basis_win: SeriesBuffer<f64>,
    #[state]
    basis_prev: Option<f64>,
    #[state]
    range_win: SeriesBuffer<f64>,
    #[state]
    range_prev: Option<f64>,
    #[state]
    previous_close: Option<f64>,
}

impl TaKcw {
    fn execute<O: PineOutput>(
        &mut self,
        ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let length = checked_length(self.length)?;
        let (high, low, close) = hlc(ctx)?;
        let previous_close = self.previous_close.replace(close);
        let range = if self.use_true_range {
            match previous_close {
                Some(pc) => (high - low).max((high - pc).abs()).max((low - pc).abs()),
                None => f64::NAN,
            }
        } else {
            high - low
        };
        let basis = ema_step(
            &mut self.basis_win,
            &mut self.basis_prev,
            self.series,
            length,
        );
        let range_ma = ema_step(&mut self.range_win, &mut self.range_prev, range, length);
        match (basis, range_ma) {
            (Some(basis), Some(range_ma)) if basis != 0.0 => {
                Ok(Value::Number(2.0 * self.mult * range_ma / basis))
            }
            (Some(_), Some(_)) => Ok(Value::Number(0.0)),
            _ => Ok(Value::Na),
        }
    }
}
