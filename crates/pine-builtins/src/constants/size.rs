use pine_interpreter::{PineOutput, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// The `size.*` constants (marker sizes for `plotshape`, `plotchar`, labels).
const SIZES: &[&str] = &["auto", "tiny", "small", "normal", "large", "huge"];

/// Register the size namespace with all size constants.
pub fn register<O: PineOutput>() -> Value<O> {
    let mut members: HashMap<String, Value<O>> = HashMap::new();

    for size in SIZES {
        members.insert(size.to_string(), Value::String(size.to_string()));
    }

    Value::Object {
        type_name: "size".to_string(),
        fields: Rc::new(RefCell::new(members)),
        call: None,
    }
}
