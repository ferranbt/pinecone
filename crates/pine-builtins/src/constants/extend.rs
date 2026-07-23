use pine_interpreter::{PineOutput, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// The `extend.*` constants (which way a line or box extends past its endpoints).
const EXTENDS: &[&str] = &["none", "left", "right", "both"];

/// Register the extend namespace with all extend constants.
pub fn register<O: PineOutput>() -> Value<O> {
    let mut members: HashMap<String, Value<O>> = HashMap::new();

    for extend in EXTENDS {
        members.insert(extend.to_string(), Value::String(extend.to_string()));
    }

    Value::Object {
        type_name: "extend".to_string(),
        fields: Rc::new(RefCell::new(members)),
        call: None,
    }
}
