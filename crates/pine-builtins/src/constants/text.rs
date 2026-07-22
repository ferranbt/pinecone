use pine_interpreter::{PineOutput, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// The `text.*` constants (text alignment and wrapping for labels/tables/boxes).
const TEXTS: &[&str] = &[
    "align_left",
    "align_center",
    "align_right",
    "align_top",
    "align_bottom",
    "wrap_none",
    "wrap_auto",
];

/// Register the text namespace with all text constants.
pub fn register<O: PineOutput>() -> Value<O> {
    let mut members: HashMap<String, Value<O>> = HashMap::new();

    for text in TEXTS {
        members.insert(text.to_string(), Value::String(text.to_string()));
    }

    Value::Object {
        type_name: "text".to_string(),
        fields: Rc::new(RefCell::new(members)),
        call: None,
    }
}
