use pine_interpreter::{PineOutput, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// The `shape.*` constants (marker shapes for `plotshape`, `plotchar`).
const SHAPES: &[&str] = &[
    "xcross",
    "cross",
    "circle",
    "triangleup",
    "triangledown",
    "flag",
    "arrowup",
    "arrowdown",
    "square",
    "diamond",
    "labelup",
    "labeldown",
];

/// Register the shape namespace with all shape constants.
pub fn register<O: PineOutput>() -> Value<O> {
    let mut members: HashMap<String, Value<O>> = HashMap::new();

    for shape in SHAPES {
        members.insert(shape.to_string(), Value::String(shape.to_string()));
    }

    Value::Object {
        type_name: "shape".to_string(),
        fields: Rc::new(RefCell::new(members)),
        call: None,
    }
}
