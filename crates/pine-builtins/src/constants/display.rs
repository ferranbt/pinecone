use pine_interpreter::{PineOutput, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// The `display.*` constants (where a plot/input is shown).
const DISPLAYS: &[&str] = &[
    "none",
    "all",
    "data_window",
    "pane",
    "price_scale",
    "status_line",
    "pine_screener",
];

/// Register the display namespace with all display constants.
pub fn register<O: PineOutput>() -> Value<O> {
    let mut members: HashMap<String, Value<O>> = HashMap::new();

    for display in DISPLAYS {
        members.insert(display.to_string(), Value::String(display.to_string()));
    }

    Value::Object {
        type_name: "display".to_string(),
        fields: Rc::new(RefCell::new(members)),
        call: None,
    }
}
