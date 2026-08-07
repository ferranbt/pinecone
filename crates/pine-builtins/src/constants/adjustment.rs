use pine_core::PineOutput;
use pine_interpreter::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// The `adjustment.*` constants (price-adjustment mode for `request.security`).
const ADJUSTMENTS: &[&str] = &["none", "splits", "dividends"];

/// Register the adjustment namespace with all its constants.
pub fn register<O: PineOutput>() -> Value<O> {
    let mut members: HashMap<String, Value<O>> = HashMap::new();

    for adjustment in ADJUSTMENTS {
        members.insert(
            adjustment.to_string(),
            Value::String(adjustment.to_string()),
        );
    }

    Value::Object {
        type_name: "adjustment".to_string(),
        fields: Rc::new(RefCell::new(members)),
        call: None,
        value: None,
    }
}
