use pine_interpreter::{PineOutput, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// The `location.*` constants (where `plotshape`/`plotchar` markers sit).
const LOCATIONS: &[&str] = &["abovebar", "belowbar", "top", "bottom", "absolute"];

/// Register the location namespace with all location constants.
pub fn register<O: PineOutput>() -> Value<O> {
    let mut members: HashMap<String, Value<O>> = HashMap::new();

    for location in LOCATIONS {
        members.insert(location.to_string(), Value::String(location.to_string()));
    }

    Value::Object {
        type_name: "location".to_string(),
        fields: Rc::new(RefCell::new(members)),
        call: None,
    }
}
