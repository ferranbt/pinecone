//! The `dividends.*` namespace: dividend-field constants and upcoming-dividend
//! variables. The forward-looking values are `na` without a fundamentals feed.

use pine_core::PineOutput;
use pine_interpreter::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// Register the `dividends.*` namespace object.
pub fn register<O: PineOutput>() -> Value<O> {
    let mut fields: HashMap<String, Value<O>> = HashMap::new();

    // Which dividend figure `request.dividends` returns.
    for constant in ["gross", "net"] {
        fields.insert(constant.to_string(), Value::String(constant.to_string()));
    }

    // Upcoming dividends, `na` without a fundamentals feed.
    for var in ["future_amount", "future_ex_date", "future_pay_date"] {
        fields.insert(var.to_string(), Value::Na);
    }

    Value::Object {
        type_name: "dividends".to_string(),
        fields: Rc::new(RefCell::new(fields)),
        call: None,
        value: None,
    }
}
