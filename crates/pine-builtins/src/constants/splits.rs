use pine_core::PineOutput;
use pine_interpreter::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// The `splits.*` constants (which field of a `request.splits` call to read).
const SPLITS: &[&str] = &["denominator", "numerator"];

/// Register the splits namespace with all splits constants.
pub fn register<O: PineOutput>() -> Value<O> {
    let mut members: HashMap<String, Value<O>> = HashMap::new();

    for split in SPLITS {
        members.insert(split.to_string(), Value::String(split.to_string()));
    }

    Value::Object {
        type_name: "splits".to_string(),
        fields: Rc::new(RefCell::new(members)),
        call: None,
    }
}
