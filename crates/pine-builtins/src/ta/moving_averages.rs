use pine_builtin_macro::BuiltinFunction;
use pine_interpreter::{Interpreter, PineOutput, RuntimeError, SeriesBuffer, Value};

/// One step of an exponentially smoothed average: `alpha * source + (1 - alpha)
/// * previous`. Pine seeds the recursion with the simple average of the first
/// `length` values, which is what `seed` carries on the very first step.
pub(crate) fn smooth_step(previous: Option<f64>, source: f64, alpha: f64, seed: &[f64]) -> f64 {
    match previous {
        Some(previous) => alpha * source + (1.0 - alpha) * previous,
        None => seed.iter().sum::<f64>() / seed.len() as f64,
    }
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

/// Rejects a zero length, which every windowed average requires.
pub(crate) fn checked_length(length: f64) -> Result<usize, RuntimeError> {
    let length = length as usize;
    if length == 0 {
        return Err(RuntimeError::TypeError(
            "length must be greater than 0".to_string(),
        ));
    }
    Ok(length)
}

/// ta.sma(source, length) - Simple Moving Average
#[derive(BuiltinFunction)]
#[builtin(name = "ta.sma", stateful)]
pub struct TaSma {
    source: f64,
    length: f64,
    #[state]
    window: SeriesBuffer<f64>,
}

impl TaSma {
    fn execute<O: PineOutput>(
        &mut self,
        _ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let length = checked_length(self.length)?;

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
        let length = checked_length(self.length)?;

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
        let length = checked_length(self.length)?;

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
    length: f64,
    #[state]
    window: SeriesBuffer<f64>,
}

impl TaWma {
    fn execute<O: PineOutput>(
        &mut self,
        _ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let length = checked_length(self.length)?;

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
        let length = checked_length(self.length)?;

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
        let length = checked_length(self.length)?;
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
