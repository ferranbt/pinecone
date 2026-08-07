use pine_core::PineOutput;
use pine_interpreter::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// The `backadjustment.*` constants (continuous-futures back-adjustment mode for
/// `request.security`).
const BACKADJUSTMENTS: &[&str] = &["inherit", "off", "on"];

/// Register the backadjustment namespace with all its constants.
pub fn register<O: PineOutput>() -> Value<O> {
    let mut members: HashMap<String, Value<O>> = HashMap::new();

    for backadjustment in BACKADJUSTMENTS {
        members.insert(
            backadjustment.to_string(),
            Value::String(backadjustment.to_string()),
        );
    }

    Value::Object {
        type_name: "backadjustment".to_string(),
        fields: Rc::new(RefCell::new(members)),
        call: None,
        value: None,
    }
}
