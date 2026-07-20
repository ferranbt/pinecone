use pine_interpreter::{PineOutput, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// The `format.*` constants (how `indicator`/`plot` values are formatted).
const FORMATS: &[&str] = &["inherit", "mintick", "percent", "price", "volume"];

/// Register the format namespace with all format constants.
pub fn register<O: PineOutput>() -> Value<O> {
    let mut members: HashMap<String, Value<O>> = HashMap::new();

    for format in FORMATS {
        members.insert(format.to_string(), Value::String(format.to_string()));
    }

    Value::Object {
        type_name: "format".to_string(),
        fields: Rc::new(RefCell::new(members)),
        call: None,
    }
}
