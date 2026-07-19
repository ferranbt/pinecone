use pine_builtin_macro::BuiltinFunction;
use pine_core::{Timeframe, TimeframeUnit};
use pine_interpreter::{DefaultPineOutput, Interpreter, RuntimeError, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

fn parse(period: &str) -> Result<Timeframe, RuntimeError> {
    Timeframe::parse(period).map_err(|e| RuntimeError::TypeError(e.to_string()))
}

/// timeframe.in_seconds(timeframe) - Length of one bar of `timeframe`, in seconds
#[derive(BuiltinFunction)]
struct TimeframeInSeconds {
    timeframe: String,
}

impl TimeframeInSeconds {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        Ok(Value::Number(parse(&self.timeframe)?.in_seconds() as f64))
    }
}

/// timeframe.from_seconds(seconds) - The timeframe whose bars last `seconds`
#[derive(BuiltinFunction)]
struct TimeframeFromSeconds {
    seconds: f64,
}

impl TimeframeFromSeconds {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        Ok(Value::String(
            Timeframe::from_seconds(self.seconds as i64).to_string(),
        ))
    }
}

/// timeframe.change(timeframe) - True on the first bar of a new `timeframe` period
#[derive(BuiltinFunction)]
struct TimeframeChange {
    timeframe: String,
}

impl TimeframeChange {
    fn execute(&self, ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        // A period boundary is where the bar's period bucket stops matching the
        // previous bar's; the first bar of all is always a boundary.
        let requested = parse(&self.timeframe)?;
        let current = requested.bucket(ctx.bar_time());
        let previous = ctx.previous_bar_time().map(|t| requested.bucket(t));

        Ok(Value::Bool(previous != Some(current)))
    }
}

/// The `timeframe` namespace for `tf`.
///
/// The members describing `tf` are plain values, since scripts read them
/// without parentheses (`timeframe.period`). Only the three real functions are
/// callable.
pub fn register(tf: Timeframe) -> Value<DefaultPineOutput> {
    let is = |unit| Value::Bool(tf.unit == unit);

    let fields: HashMap<String, Value<DefaultPineOutput>> = [
        ("period", Value::String(tf.to_string())),
        // Outside a `request.*` call the main period is the period.
        ("main_period", Value::String(tf.to_string())),
        ("multiplier", Value::Number(tf.multiplier as f64)),
        ("isticks", is(TimeframeUnit::Tick)),
        ("isseconds", is(TimeframeUnit::Second)),
        ("isminutes", is(TimeframeUnit::Minute)),
        ("isdaily", is(TimeframeUnit::Day)),
        ("isweekly", is(TimeframeUnit::Week)),
        ("ismonthly", is(TimeframeUnit::Month)),
        ("isintraday", Value::Bool(tf.is_intraday())),
        ("isdwm", Value::Bool(tf.is_dwm())),
        (
            "in_seconds",
            Value::BuiltinFunction(Rc::new(TimeframeInSeconds::builtin_fn)),
        ),
        (
            "from_seconds",
            Value::BuiltinFunction(Rc::new(TimeframeFromSeconds::builtin_fn)),
        ),
        (
            "change",
            Value::BuiltinFunction(Rc::new(TimeframeChange::builtin_fn)),
        ),
    ]
    .into_iter()
    .map(|(name, value)| (name.to_string(), value))
    .collect();

    Value::Object {
        type_name: "timeframe".to_string(),
        fields: Rc::new(RefCell::new(fields)),
        call: None,
    }
}
