use pine_core::PineOutput;
use pine_interpreter::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// The `settlement_as_close.*` constants (whether futures use the settlement
/// price as the close, for `request.security`).
const SETTLEMENTS: &[&str] = &["inherit", "off", "on"];

/// Register the settlement_as_close namespace with all its constants.
pub fn register<O: PineOutput>() -> Value<O> {
    let mut members: HashMap<String, Value<O>> = HashMap::new();

    for settlement in SETTLEMENTS {
        members.insert(settlement.to_string(), Value::String(settlement.to_string()));
    }

    Value::Object {
        type_name: "settlement_as_close".to_string(),
        fields: Rc::new(RefCell::new(members)),
        call: None,
    }
}
