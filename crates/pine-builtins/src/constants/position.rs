use pine_interpreter::{PineOutput, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// The `position.*` constants (anchor for `table.new`).
const POSITIONS: &[&str] = &[
    "top_left",
    "top_center",
    "top_right",
    "middle_left",
    "middle_center",
    "middle_right",
    "bottom_left",
    "bottom_center",
    "bottom_right",
    "top",
    "middle",
    "bottom",
];

/// Register the position namespace with all position constants.
pub fn register<O: PineOutput>() -> Value<O> {
    let mut members: HashMap<String, Value<O>> = HashMap::new();

    for position in POSITIONS {
        members.insert(position.to_string(), Value::String(position.to_string()));
    }

    Value::Object {
        type_name: "position".to_string(),
        fields: Rc::new(RefCell::new(members)),
        call: None,
    }
}
