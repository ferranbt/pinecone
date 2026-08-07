//! The `earnings.*` namespace: earnings-field constants and upcoming-earnings
//! variables. The forward-looking values are `na` without a fundamentals feed.

use pine_core::PineOutput;
use pine_interpreter::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// Register the `earnings.*` namespace object.
pub fn register<O: PineOutput>() -> Value<O> {
    let mut fields: HashMap<String, Value<O>> = HashMap::new();

    // Which earnings figure `request.earnings` returns.
    for constant in ["actual", "estimate", "standardized"] {
        fields.insert(constant.to_string(), Value::String(constant.to_string()));
    }

    // Upcoming earnings, `na` without a fundamentals feed.
    for var in ["future_eps", "future_revenue", "future_time", "future_period_end_time"] {
        fields.insert(var.to_string(), Value::Na);
    }

    Value::Object {
        type_name: "earnings".to_string(),
        fields: Rc::new(RefCell::new(fields)),
        call: None,
    }
}
