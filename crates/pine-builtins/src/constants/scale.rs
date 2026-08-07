use pine_core::PineOutput;
use pine_interpreter::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// The `scale.*` constants (which price scale an indicator is attached to).
const SCALES: &[&str] = &["left", "right", "none"];

/// Register the scale namespace with all scale constants.
pub fn register<O: PineOutput>() -> Value<O> {
    let mut members: HashMap<String, Value<O>> = HashMap::new();

    for scale in SCALES {
        members.insert(scale.to_string(), Value::String(scale.to_string()));
    }

    Value::Object {
        type_name: "scale".to_string(),
        fields: Rc::new(RefCell::new(members)),
        call: None,
    }
}
