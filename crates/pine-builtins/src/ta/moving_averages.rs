use pine_builtin_macro::BuiltinFunction;
use pine_core::{PineOutput, SeriesBuffer};
use pine_interpreter::{Interpreter, RuntimeError, Value};

/// One step of an exponentially smoothed average: `alpha * source + (1 - alpha)
/// * previous`. Pine seeds the recursion with the simple average of the first
/// `length` values, which is what `seed` carries on the very first step.
pub(crate) fn smooth_step(previous: Option<f64>, source: f64, alpha: f64, seed: &[f64]) -> f64 {
    match previous {
        Some(previous) => alpha * source + (1.0 - alpha) * previous,
        None => seed.iter().sum::<f64>() / seed.len() as f64,
    }
}

/// One EMA step over a persistent `(window, previous)` pair: seeds with the SMA
/// of the first `length` values, then recurses with `alpha = 2 / (length + 1)`.
/// An `na` source holds the last value without advancing the seed, which is how
/// Pine composes an EMA over a series that starts `na` (e.g. the range in
/// `ta.kc`).
pub(crate) fn ema_step(
    window: &mut SeriesBuffer<f64>,
    previous: &mut Option<f64>,
    source: f64,
    length: usize,
) -> Option<f64> {
    if source.is_nan() {
        return *previous;
    }
    let seed = window.observe(source, length)?;
    let alpha = 2.0 / (length as f64 + 1.0);
    let ema = smooth_step(*previous, source, alpha, &seed);
    *previous = Some(ema);
    Some(ema)
}

/// One Wilder (RMA) step over a persistent `(window, previous)` pair: seeds with
/// the SMA of the first `length` values, then recurses with `alpha = 1 / length`.
/// An `na` source holds the last value without advancing the seed.
pub(crate) fn wilder_step(
    window: &mut SeriesBuffer<f64>,
    previous: &mut Option<f64>,
    source: f64,
    length: usize,
) -> Option<f64> {
    if source.is_nan() {
        return *previous;
    }
    let seed = window.observe(source, length)?;
    let alpha = 1.0 / length as f64;
    let value = smooth_step(*previous, source, alpha, &seed);
    *previous = Some(value);
    Some(value)
}

/// Weighted average of `values` (newest first), weighting the newest highest:
/// `n, n-1, … 1`.
pub(crate) fn weighted_average(values: &[f64]) -> f64 {
    let len = values.len();
    let weighted: f64 = values
        .iter()
        .enumerate()
        .map(|(i, &value)| value * (len - i) as f64)
        .sum();
    let total_weight = (len * (len + 1)) as f64 / 2.0;
    weighted / total_weight
}

/// ta.sma(source, length) - Simple Moving Average
#[derive(BuiltinFunction)]
#[builtin(name = "ta.sma", stateful)]
pub struct TaSma {
    source: f64,
    #[length_check]
    length: f64,
    #[state]
    window: SeriesBuffer<f64>,
}

impl TaSma {
    fn execute<O: PineOutput>(
        &mut self,
        _ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let length = self.length as usize;

        let Some(values) = self.window.observe(self.source, length) else {
            return Ok(Value::Na);
        };

        Ok(Value::Number(values.iter().sum::<f64>() / length as f64))
    }
}

/// ta.ema(source, length) - Exponential Moving Average
///
/// `alpha * source + (1 - alpha) * ema[1]` with `alpha = 2 / (length + 1)`,
/// carried across bars. Like Pine, it is na until `length` bars have been seen
/// and then starts from their simple average.
#[derive(BuiltinFunction)]
#[builtin(name = "ta.ema", stateful)]
pub struct TaEma {
    source: f64,
    #[length_check]
    length: f64,
    /// Holds the first `length` values, which seed the recursion.
    #[state]
    window: SeriesBuffer<f64>,
    #[state]
    previous: Option<f64>,
}

impl TaEma {
    fn execute<O: PineOutput>(
        &mut self,
        _ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let length = self.length as usize;

        let Some(seed) = self.window.observe(self.source, length) else {
            return Ok(Value::Na);
        };

        let alpha = 2.0 / (length as f64 + 1.0);
        let ema = smooth_step(self.previous, self.source, alpha, &seed);
        self.previous = Some(ema);

        Ok(Value::Number(ema))
    }
}

/// ta.rma(source, length) - Rolling Moving Average (Wilder's Smoothing)
///
/// The same recursion as [`TaEma`] with `alpha = 1 / length`, which smooths more
/// slowly. This is what `ta.rsi` averages its gains and losses with.
#[derive(BuiltinFunction)]
#[builtin(name = "ta.rma", stateful)]
pub struct TaRma {
    source: f64,
    #[length_check]
    length: f64,
    #[state]
    window: SeriesBuffer<f64>,
    #[state]
    previous: Option<f64>,
}

impl TaRma {
    fn execute<O: PineOutput>(
        &mut self,
        _ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let length = self.length as usize;

        // `na` inputs are ignored: they neither seed the window nor advance the
        // recursion, so the average is over `length` non-`na` values (v6 spec).
        if self.source.is_nan() {
            return Ok(Value::Na);
        }

        let Some(seed) = self.window.observe(self.source, length) else {
            return Ok(Value::Na);
        };

        let rma = smooth_step(self.previous, self.source, 1.0 / length as f64, &seed);
        self.previous = Some(rma);

        Ok(Value::Number(rma))
    }
}

/// ta.wma(source, length) - Weighted Moving Average
#[derive(BuiltinFunction)]
#[builtin(name = "ta.wma", stateful)]
pub struct TaWma {
    source: f64,
    #[length_check]
    length: f64,
    #[state]
    window: SeriesBuffer<f64>,
}

impl TaWma {
    fn execute<O: PineOutput>(
        &mut self,
        _ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let length = self.length as usize;

        let Some(values) = self.window.observe(self.source, length) else {
            return Ok(Value::Na);
        };

        Ok(Value::Number(weighted_average(&values)))
    }
}

/// ta.vwma(source, length) - Volume Weighted Moving Average
#[derive(BuiltinFunction)]
#[builtin(name = "ta.vwma", stateful)]
pub struct TaVwma {
    source: f64,
    #[length_check]
    length: f64,
    #[state]
    prices: SeriesBuffer<f64>,
    #[state]
    volumes: SeriesBuffer<f64>,
}

impl TaVwma {
    fn execute<O: PineOutput>(
        &mut self,
        ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let length = self.length as usize;

        let volume = ctx
            .get_variable("volume")
            .ok_or_else(|| RuntimeError::UndefinedVariable("volume".to_string()))?
            .as_number()?;

        // Both sides advance together, so they fill on the same bar.
        let prices = self.prices.observe(self.source, length);
        let volumes = self.volumes.observe(volume, length);
        let (Some(prices), Some(volumes)) = (prices, volumes) else {
            return Ok(Value::Na);
        };

        let volume_sum: f64 = volumes.iter().sum();
        if volume_sum == 0.0 {
            return Ok(Value::Na);
        }

        let weighted: f64 = prices
            .iter()
            .zip(&volumes)
            .map(|(price, volume)| price * volume)
            .sum();

        Ok(Value::Number(weighted / volume_sum))
    }
}

/// ta.hma(source, length) - Hull Moving Average
///
/// `wma(2 * wma(source, length/2) - wma(source, length), sqrt(length))`. The
/// outer average needs the inner one's own history, so it keeps a second buffer.
#[derive(BuiltinFunction)]
#[builtin(name = "ta.hma", stateful)]
pub struct TaHma {
    source: f64,
    #[length_check]
    length: f64,
    #[state]
    window: SeriesBuffer<f64>,
    /// Past values of `2 * wma(length/2) - wma(length)`, which the outer
    /// weighted average smooths.
    #[state]
    raw: SeriesBuffer<f64>,
}

impl TaHma {
    fn execute<O: PineOutput>(
        &mut self,
        _ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let length = self.length as usize;
        let half = (length / 2).max(1);
        let root = ((length as f64).sqrt().floor() as usize).max(1);

        let Some(values) = self.window.observe(self.source, length) else {
            return Ok(Value::Na);
        };

        let raw = 2.0 * weighted_average(&values[..half]) - weighted_average(&values);

        let Some(smoothed) = self.raw.observe(raw, root) else {
            return Ok(Value::Na);
        };

        Ok(Value::Number(weighted_average(&smoothed)))
    }
}

/// ta.swma(source) - Symmetrically Weighted Moving Average
///
/// A fixed 4-bar average weighted `1, 2, 2, 1` from newest to oldest.
#[derive(BuiltinFunction)]
#[builtin(name = "ta.swma", stateful)]
pub struct TaSwma {
    source: f64,
    #[state]
    window: SeriesBuffer<f64>,
}

impl TaSwma {
    fn execute<O: PineOutput>(
        &mut self,
        _ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let Some(values) = self.window.observe(self.source, 4) else {
            return Ok(Value::Na);
        };

        let swma = (values[0] + 2.0 * values[1] + 2.0 * values[2] + values[3]) / 6.0;
        Ok(Value::Number(swma))
    }
}

/// ta.alma(series, length, offset, sigma) - Arnaud Legoux Moving Average.
///
/// A Gaussian-weighted average of the last `length` values; `offset` (`0…1`)
/// slides the weight peak toward the newest bar and `sigma` sets its spread.
#[derive(BuiltinFunction)]
#[builtin(name = "ta.alma", stateful)]
pub struct TaAlma {
    series: f64,
    #[length_check]
    length: f64,
    offset: f64,
    sigma: f64,
    #[state]
    window: SeriesBuffer<f64>,
}

impl TaAlma {
    fn execute<O: PineOutput>(
        &mut self,
        _ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let length = self.length as usize;
        let Some(values) = self.window.observe(self.series, length) else {
            return Ok(Value::Na);
        };
        let m = self.offset * (length as f64 - 1.0);
        let s = length as f64 / self.sigma;
        let weights: Vec<f64> = (0..length)
            .map(|w| (-((w as f64 - m).powi(2)) / (2.0 * s * s)).exp())
            .collect();
        let norm: f64 = weights.iter().sum();
        // `values` is newest-first, so weight `w` pairs with `values[length-1-w]`.
        let sum: f64 = weights
            .iter()
            .enumerate()
            .map(|(w, weight)| weight * values[length - 1 - w])
            .sum();
        Ok(Value::Number(sum / norm))
    }
}
