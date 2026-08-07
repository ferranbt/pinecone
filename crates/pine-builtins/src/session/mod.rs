//! The `session.*` namespace: session-type constants and session-state variables.
//!
//! Headless there is no intraday session calendar, so the state variables take
//! sensible defaults — the bar is treated as inside the regular market session.

use pine_core::PineOutput;
use pine_interpreter::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// Register the `session.*` namespace object.
pub fn register<O: PineOutput>() -> Value<O> {
    let mut fields: HashMap<String, Value<O>> = HashMap::new();

    // Session-type constants (arguments to `ticker.new`/`request.security`).
    fields.insert("regular".to_string(), Value::String("regular".to_string()));
    fields.insert("extended".to_string(), Value::String("extended".to_string()));

    // Session-state variables: treat every bar as regular-session, first/last off.
    fields.insert("ismarket".to_string(), Value::Bool(true));
    for off in [
        "ispremarket",
        "ispostmarket",
        "isfirstbar",
        "isfirstbar_regular",
        "islastbar",
        "islastbar_regular",
    ] {
        fields.insert(off.to_string(), Value::Bool(false));
    }

    Value::Object {
        type_name: "session".to_string(),
        fields: Rc::new(RefCell::new(fields)),
        call: None,
    }
}
