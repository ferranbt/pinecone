use pine_interpreter::{PineOutput, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// The `barmerge.*` constants (how requested data is aligned to chart bars).
const BARMERGES: &[&str] = &["gaps_on", "gaps_off", "lookahead_on", "lookahead_off"];

/// Register the barmerge namespace with all barmerge constants.
pub fn register<O: PineOutput>() -> Value<O> {
    let mut members: HashMap<String, Value<O>> = HashMap::new();

    for barmerge in BARMERGES {
        members.insert(barmerge.to_string(), Value::String(barmerge.to_string()));
    }

    Value::Object {
        type_name: "barmerge".to_string(),
        fields: Rc::new(RefCell::new(members)),
        call: None,
    }
}
