use pine_core::PineOutput;
use pine_interpreter::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// The `yloc.*` constants (vertical placement for labels/lines relative to a bar).
const YLOCS: &[&str] = &["abovebar", "belowbar", "price"];

/// Register the yloc namespace with all yloc constants.
pub fn register<O: PineOutput>() -> Value<O> {
    let mut members: HashMap<String, Value<O>> = HashMap::new();

    for yloc in YLOCS {
        members.insert(yloc.to_string(), Value::String(yloc.to_string()));
    }

    Value::Object {
        type_name: "yloc".to_string(),
        fields: Rc::new(RefCell::new(members)),
        call: None,
        value: None,
    }
}
