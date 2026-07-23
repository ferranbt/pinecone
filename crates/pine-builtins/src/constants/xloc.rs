use pine_interpreter::{PineOutput, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// The `xloc.*` constants (how a drawing's x coordinates are interpreted).
const XLOCS: &[&str] = &["bar_index", "bar_time"];

/// Register the xloc namespace with all xloc constants.
pub fn register<O: PineOutput>() -> Value<O> {
    let mut members: HashMap<String, Value<O>> = HashMap::new();

    for xloc in XLOCS {
        members.insert(xloc.to_string(), Value::String(xloc.to_string()));
    }

    Value::Object {
        type_name: "xloc".to_string(),
        fields: Rc::new(RefCell::new(members)),
        call: None,
    }
}
