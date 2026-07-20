use pine_core::Timeframe;
use pine_interpreter::{PineOutput, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// Build the `timeframe` namespace object from the host-supplied timeframe.
///
/// Exposes the data members `timeframe.period` / `timeframe.multiplier` and the
/// `timeframe.is*` classification flags, all derived from the period string. The
/// `timeframe.*` functions (`in_seconds`, `change`, ...) are not registered yet.
pub fn register<O: PineOutput>(tf: Timeframe) -> Value<O> {
    let mut members: HashMap<String, Value<O>> = HashMap::new();

    members.insert("period".to_string(), Value::String(tf.period()));
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
