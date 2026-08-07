use pine_builtin_macro::BuiltinFunction;
use pine_core::{PineOutput, Timeframe};
use pine_interpreter::{Interpreter, RuntimeError, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// timeframe.in_seconds(timeframe) - The length of a timeframe in seconds
/// (defaults to the chart timeframe when the string is empty).
#[derive(BuiltinFunction)]
#[builtin(name = "timeframe.in_seconds")]
struct TimeframeInSeconds {
    #[arg(default = "")]
    timeframe: String,
}

impl TimeframeInSeconds {
    fn execute<O: PineOutput>(&self, ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let millis = if self.timeframe.is_empty() {
            ctx.chart_period
        } else {
            self.timeframe
                .parse::<Timeframe>()
                .ok()
                .and_then(|tf| tf.to_millis())
        };
        Ok(millis.map_or(Value::Na, |ms| Value::Int(ms / 1000)))
    }
}

/// timeframe.from_seconds(seconds) - The timeframe string for a number of
/// seconds.
#[derive(BuiltinFunction)]
#[builtin(name = "timeframe.from_seconds")]
struct TimeframeFromSeconds {
    seconds: f64,
}

impl TimeframeFromSeconds {
    fn execute<O: PineOutput>(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let millis = (self.seconds * 1000.0) as i64;
        Ok(Timeframe::from_millis(millis).map_or(Value::Na, |tf| Value::String(tf.period())))
    }
}

/// Build the `timeframe` namespace object from the host-supplied timeframe.
///
/// Exposes the data members `timeframe.period` / `timeframe.multiplier` and the
/// `timeframe.is*` classification flags, all derived from the period string. The
/// `timeframe.*` functions (`in_seconds`, `change`, ...) are not registered yet.
pub fn register<O: PineOutput>(tf: Timeframe) -> Value<O> {
    let mut members: HashMap<String, Value<O>> = HashMap::new();

    members.insert("period".to_string(), Value::String(tf.period()));
    members.insert("main_period".to_string(), Value::String(tf.period()));
    members.insert(
        "in_seconds".to_string(),
        TimeframeInSeconds::builtin_value::<O>(),
    );
    members.insert(
        "from_seconds".to_string(),
        TimeframeFromSeconds::builtin_value::<O>(),
    );
    members.insert(
        "multiplier".to_string(),
        Value::Number(tf.multiplier as f64),
    );
    members.insert("isseconds".to_string(), Value::Bool(tf.is_seconds()));
    members.insert("isminutes".to_string(), Value::Bool(tf.is_minutes()));
    members.insert("isintraday".to_string(), Value::Bool(tf.is_intraday()));
    members.insert("isdaily".to_string(), Value::Bool(tf.is_daily()));
    members.insert("isweekly".to_string(), Value::Bool(tf.is_weekly()));
    members.insert("ismonthly".to_string(), Value::Bool(tf.is_monthly()));
    members.insert("isdwm".to_string(), Value::Bool(tf.is_dwm()));
    members.insert("isticks".to_string(), Value::Bool(tf.is_ticks()));

    Value::Object {
        type_name: "timeframe".to_string(),
        fields: Rc::new(RefCell::new(members)),
        call: None,
    }
}
