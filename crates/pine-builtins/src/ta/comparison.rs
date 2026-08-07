use super::moving_averages::checked_length;
use pine_builtin_macro::BuiltinFunction;
use pine_core::{PineOutput, SeriesBuffer};
use pine_interpreter::{Interpreter, RuntimeError, Value};

/// The current-bar value of a built-in price series (`high`, `low`, …). Used as
/// the default `source` for the one-argument overloads.
fn bar_source<O: PineOutput>(ctx: &Interpreter<O>, name: &str) -> f64 {
    ctx.get_variable(name)
        .and_then(|value| value.as_number().ok())
        .unwrap_or(f64::NAN)
}

/// ta.change(source, length) - Difference between current and N bars ago
#[derive(BuiltinFunction)]
#[builtin(name = "ta.change", stateful)]
pub struct TaChange {
    source: f64,
    #[arg(default = 1.0)]
    length: f64,
    #[state]
    window: SeriesBuffer<f64>,
}

impl TaChange {
    fn execute<O: PineOutput>(
        &mut self,
        _ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let length = self.length as usize;
        if length == 0 {
            return Ok(Value::Number(0.0));
        }

        // Reaching `length` bars back needs `length + 1` values in hand.
        let Some(values) = self.window.observe(self.source, length + 1) else {
            return Ok(Value::Na);
        };

        Ok(Value::Number(values[0] - values[length]))
    }
}

/// ta.highest(source, length) - Highest value over N bars
#[derive(BuiltinFunction)]
#[builtin(name = "ta.highest", stateful)]
pub struct TaHighest {
    #[arg(default = bar_source(ctx, "high"))]
    source: f64,
    length: f64,
    #[state]
    window: SeriesBuffer<f64>,
}

impl TaHighest {
    fn execute<O: PineOutput>(
        &mut self,
        _ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let length = checked_length(self.length)?;

        let Some(values) = self.window.observe(self.source, length) else {
            return Ok(Value::Na);
        };

        Ok(Value::Number(
            values.iter().copied().fold(f64::NEG_INFINITY, f64::max),
        ))
    }
}

/// ta.lowest(source, length) - Lowest value over N bars
#[derive(BuiltinFunction)]
#[builtin(name = "ta.lowest", stateful)]
pub struct TaLowest {
    #[arg(default = bar_source(ctx, "low"))]
    source: f64,
    length: f64,
    #[state]
    window: SeriesBuffer<f64>,
}

impl TaLowest {
    fn execute<O: PineOutput>(
        &mut self,
        _ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let length = checked_length(self.length)?;

        let Some(values) = self.window.observe(self.source, length) else {
            return Ok(Value::Na);
        };

        Ok(Value::Number(
            values.iter().copied().fold(f64::INFINITY, f64::min),
        ))
    }
}

/// The offset of the extreme value in `values` (newest first), as the negative
/// bar count Pine reports: 0 is the current bar, -1 the one before it.
fn extreme_offset(values: &[f64], better: fn(f64, f64) -> bool) -> f64 {
    let mut best = 0;
    for (i, &value) in values.iter().enumerate() {
        if better(value, values[best]) {
            best = i;
        }
    }
    // Negating 0 would give `-0`, which prints as "-0" rather than "0".
    if best == 0 {
        0.0
    } else {
        -(best as f64)
    }
}

/// ta.highestbars(source, length) - Offset to highest value (0 = current bar)
#[derive(BuiltinFunction)]
#[builtin(name = "ta.highestbars", stateful)]
pub struct TaHighestbars {
    #[arg(default = bar_source(ctx, "high"))]
    source: f64,
    length: f64,
    #[state]
    window: SeriesBuffer<f64>,
}

impl TaHighestbars {
    fn execute<O: PineOutput>(
        &mut self,
        _ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let length = checked_length(self.length)?;

        let Some(values) = self.window.observe(self.source, length) else {
            return Ok(Value::Na);
        };

        Ok(Value::Number(extreme_offset(&values, |a, b| a > b)))
    }
}

/// ta.lowestbars(source, length) - Offset to lowest value (0 = current bar)
#[derive(BuiltinFunction)]
#[builtin(name = "ta.lowestbars", stateful)]
pub struct TaLowestbars {
    #[arg(default = bar_source(ctx, "low"))]
    source: f64,
    length: f64,
    #[state]
    window: SeriesBuffer<f64>,
}

impl TaLowestbars {
    fn execute<O: PineOutput>(
        &mut self,
        _ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let length = checked_length(self.length)?;

        let Some(values) = self.window.observe(self.source, length) else {
            return Ok(Value::Na);
        };

        Ok(Value::Number(extreme_offset(&values, |a, b| a < b)))
    }
}

/// ta.rising(source, length) - Test if source rose on each of the last N bars
#[derive(BuiltinFunction)]
#[builtin(name = "ta.rising", stateful)]
pub struct TaRising {
    source: f64,
    length: f64,
    #[state]
    window: SeriesBuffer<f64>,
}

impl TaRising {
    fn execute<O: PineOutput>(
        &mut self,
        _ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let length = self.length as usize;
        if length == 0 {
            return Ok(Value::Bool(false));
        }

        let Some(values) = self.window.observe(self.source, length + 1) else {
            return Ok(Value::Bool(false));
        };

        Ok(Value::Bool(values.windows(2).all(|pair| pair[0] > pair[1])))
    }
}

/// ta.falling(source, length) - Test if source fell on each of the last N bars
#[derive(BuiltinFunction)]
#[builtin(name = "ta.falling", stateful)]
pub struct TaFalling {
    source: f64,
    length: f64,
    #[state]
    window: SeriesBuffer<f64>,
}

impl TaFalling {
    fn execute<O: PineOutput>(
        &mut self,
        _ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let length = self.length as usize;
        if length == 0 {
            return Ok(Value::Bool(false));
        }

        let Some(values) = self.window.observe(self.source, length + 1) else {
            return Ok(Value::Bool(false));
        };

        Ok(Value::Bool(values.windows(2).all(|pair| pair[0] < pair[1])))
    }
}

/// How two series sat relative to each other on this bar and the last, which is
/// all the cross tests need.
struct Crossing {
    now_above: bool,
    was_above: bool,
}

impl Crossing {
    /// `None` until both series have two bars of history.
    fn observe(
        first: &mut SeriesBuffer<f64>,
        second: &mut SeriesBuffer<f64>,
        source1: f64,
        source2: f64,
    ) -> Option<Self> {
        // Both sides advance together, so they fill on the same bar.
        let firsts = first.observe(source1, 2);
        let seconds = second.observe(source2, 2);
        let (firsts, seconds) = (firsts?, seconds?);

        Some(Self {
            now_above: firsts[0] > seconds[0],
            was_above: firsts[1] > seconds[1],
        })
    }
}

/// ta.cross(source1, source2) - Test if two series crossed in either direction
#[derive(BuiltinFunction)]
#[builtin(name = "ta.cross", stateful)]
pub struct TaCross {
    source1: f64,
    source2: f64,
    #[state]
    first: SeriesBuffer<f64>,
    #[state]
    second: SeriesBuffer<f64>,
}

impl TaCross {
    fn execute<O: PineOutput>(
        &mut self,
        _ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let Some(crossing) = Crossing::observe(
            &mut self.first,
            &mut self.second,
            self.source1,
            self.source2,
        ) else {
            return Ok(Value::Bool(false));
        };

        Ok(Value::Bool(crossing.now_above != crossing.was_above))
    }
}

/// ta.crossover(source1, source2) - Test if source1 crossed above source2
#[derive(BuiltinFunction)]
#[builtin(name = "ta.crossover", stateful)]
pub struct TaCrossover {
    source1: f64,
    source2: f64,
    #[state]
    first: SeriesBuffer<f64>,
    #[state]
    second: SeriesBuffer<f64>,
}

impl TaCrossover {
    fn execute<O: PineOutput>(
        &mut self,
        _ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let Some(crossing) = Crossing::observe(
            &mut self.first,
            &mut self.second,
            self.source1,
            self.source2,
        ) else {
            return Ok(Value::Bool(false));
        };

        Ok(Value::Bool(crossing.now_above && !crossing.was_above))
    }
}

/// ta.crossunder(source1, source2) - Test if source1 crossed below source2
#[derive(BuiltinFunction)]
#[builtin(name = "ta.crossunder", stateful)]
pub struct TaCrossunder {
    source1: f64,
    source2: f64,
    #[state]
    first: SeriesBuffer<f64>,
    #[state]
    second: SeriesBuffer<f64>,
}

impl TaCrossunder {
    fn execute<O: PineOutput>(
        &mut self,
        _ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let Some(crossing) = Crossing::observe(
            &mut self.first,
            &mut self.second,
            self.source1,
            self.source2,
        ) else {
            return Ok(Value::Bool(false));
        };

        Ok(Value::Bool(!crossing.now_above && crossing.was_above))
    }
}

/// ta.barssince(condition) - Bars since `condition` was last true (`0` on the
/// bar it is true); `na` until it has been true at least once.
#[derive(BuiltinFunction)]
#[builtin(name = "ta.barssince", stateful)]
pub struct TaBarssince {
    condition: bool,
    #[state]
    since: Option<u64>,
}

impl TaBarssince {
    fn execute<O: PineOutput>(
        &mut self,
        _ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        self.since = if self.condition {
            Some(0)
        } else {
            self.since.map(|s| s + 1)
        };
        Ok(self.since.map_or(Value::Na, |s| Value::Number(s as f64)))
    }
}

/// ta.valuewhen(condition, source, occurrence) - The value of `source` when
/// `condition` was true `occurrence` times ago (`0` is the most recent).
#[derive(BuiltinFunction)]
#[builtin(name = "ta.valuewhen", stateful)]
pub struct TaValuewhen {
    condition: bool,
    source: f64,
    occurrence: f64,
    #[state]
    values: Vec<f64>,
}

impl TaValuewhen {
    fn execute<O: PineOutput>(
        &mut self,
        _ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        if self.condition {
            self.values.push(self.source);
        }
        if self.occurrence < 0.0 || self.occurrence.is_nan() {
            return Ok(Value::Na);
        }
        let occurrence = self.occurrence as usize;
        let Some(index) = self.values.len().checked_sub(occurrence + 1) else {
            return Ok(Value::Na);
        };
        Ok(Value::Number(self.values[index]))
    }
}

/// The pivot value `rightbars` bars back if it is the strict extreme of the
/// `leftbars + 1 + rightbars` window, else `None`. `strict_max` picks a high pivot.
fn pivot(window: &[f64], rightbars: usize, strict_max: bool) -> Option<f64> {
    let candidate = window[rightbars];
    let is_pivot = window.iter().enumerate().all(|(i, &v)| {
        i == rightbars
            || if strict_max {
                candidate > v
            } else {
                candidate < v
            }
    });
    is_pivot.then_some(candidate)
}

/// ta.pivothigh(source, leftbars, rightbars) - The pivot high `rightbars` bars
/// back (`source` defaults to `high`).
#[derive(BuiltinFunction)]
#[builtin(name = "ta.pivothigh", stateful)]
pub struct TaPivothigh {
    #[arg(default = bar_source(ctx, "high"))]
    source: f64,
    leftbars: f64,
    rightbars: f64,
    #[state]
    window: SeriesBuffer<f64>,
}

impl TaPivothigh {
    fn execute<O: PineOutput>(
        &mut self,
        _ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let (left, right) = (self.leftbars as usize, self.rightbars as usize);
        let Some(window) = self.window.observe(self.source, left + right + 1) else {
            return Ok(Value::Na);
        };
        Ok(pivot(&window, right, true).map_or(Value::Na, Value::Number))
    }
}

/// ta.pivotlow(source, leftbars, rightbars) - The pivot low `rightbars` bars back
/// (`source` defaults to `low`).
#[derive(BuiltinFunction)]
#[builtin(name = "ta.pivotlow", stateful)]
pub struct TaPivotlow {
    #[arg(default = bar_source(ctx, "low"))]
    source: f64,
    leftbars: f64,
    rightbars: f64,
    #[state]
    window: SeriesBuffer<f64>,
}

impl TaPivotlow {
    fn execute<O: PineOutput>(
        &mut self,
        _ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let (left, right) = (self.leftbars as usize, self.rightbars as usize);
        let Some(window) = self.window.observe(self.source, left + right + 1) else {
            return Ok(Value::Na);
        };
        Ok(pivot(&window, right, false).map_or(Value::Na, Value::Number))
    }
}
