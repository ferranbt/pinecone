use super::moving_averages::{checked_length, smooth_step};
use pine_builtin_macro::BuiltinFunction;
use pine_interpreter::{Interpreter, PineOutput, RuntimeError, SeriesBuffer, Value};
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
