use pine_interpreter::{PineOutput, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// The `order.*` constants (sort direction for `array.sort`, matrix ops, ...).
const ORDERS: &[&str] = &["ascending", "descending", "alert", "alert_message"];

/// Register the order namespace with all order constants.
pub fn register<O: PineOutput>() -> Value<O> {
    let mut members: HashMap<String, Value<O>> = HashMap::new();

    for order in ORDERS {
        members.insert(order.to_string(), Value::String(order.to_string()));
    }

    Value::Object {
        type_name: "order".to_string(),
        fields: Rc::new(RefCell::new(members)),
        call: None,
    }
}
